pub mod binance;
pub mod errors;
pub mod poloniex;

use std::fmt::Debug;

use async_trait::async_trait;
use futures::StreamExt;
use hmac::Hmac;
use rotom_data::{
    error::SocketError,
    protocols::ws::{
        ws_parser::{StreamParser, WebSocketParser},
        JoinHandle, WsRead,
    },
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::portfolio::OrderEvent;

/*----- */
// Convenient types
/*----- */
type HmacSha256 = Hmac<Sha256>;

/*----- */
// Private Data Ws Stream
/*----- */
pub struct UserDataStream {
    pub user_data_ws: WsRead,
    pub tasks: Option<Vec<JoinHandle>>,
}

impl UserDataStream {
    pub fn cancel_running_tasks(self) {
        if let Some(tasks) = self.tasks {
            tasks.iter().for_each(|task| {
                task.abort();
            });
        }
    }
}

/*----- */
// Execution Client Trait
/*----- */
#[async_trait]
pub trait ExecutionClient2 {
    const CLIENT: ExecutionId;

    type CancelResponse;
    type CancelAllResponse;
    type NewOrderResponse;
    type WalletTransferResponse;
    type UserDataStreamResponse;

    // Init user data websocket
    async fn account_data_ws_init() -> Result<UserDataStream, SocketError>;

    // Init exchange executor
    fn http_client_init() -> Result<Self, SocketError>
    where
        Self: Sized;

    // Open order for single asset
    async fn open_order(
        &self,
        open_requests: OrderEvent,
    ) -> Result<Self::NewOrderResponse, SocketError>;

    // Cancel order for a single asset
    async fn cancel_order(
        &self,
        order_id: String,
        symbol: String,
    ) -> Result<Self::CancelResponse, SocketError>;

    // Cancel all orders for a single asset
    async fn cancel_order_all(
        &self,
        symbol: String,
    ) -> Result<Self::CancelAllResponse, SocketError>;

    // Transfer to another wallet
    async fn wallet_transfer(
        &self,
        coin: String,
        wallet_address: String,
        network: Option<String>,
        amount: f64,
    ) -> Result<Self::WalletTransferResponse, SocketError>;
}


/*----- */
// Execution ID
/*----- */
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(rename = "execution", rename_all = "snake_case")]
pub enum ExecutionId {
    Poloniex,
    Binance,
}

impl std::fmt::Display for ExecutionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionId::Binance => write!(f, "binance"),
            ExecutionId::Poloniex => write!(f, "poloniex"),
        }
    }
}

/*----- */
// Function to spawn and return a tx for private user ws
/*----- */
pub async fn consume_account_data_ws<ExchangeClient>(
    account_data_tx: mpsc::UnboundedSender<ExchangeClient::UserDataStreamResponse>,
) -> Result<(), SocketError>
where
    ExchangeClient: ExecutionClient2,
    ExchangeClient::UserDataStreamResponse: for<'de> Deserialize<'de> + Send + Debug + 'static,
{
    let exchange_id = ExchangeClient::CLIENT;
    let mut connection_attempt: u32 = 0;
    let mut _backoff_ms: u64 = 125;

    info!(
        exchange = %exchange_id,
        action = "Connecting to private user websocket stream"
    );

    loop {
        let mut stream = ExchangeClient::account_data_ws_init().await?;
        connection_attempt += 1;
        _backoff_ms *= 2;

        while let Some(msg) = stream.user_data_ws.next().await {
            match WebSocketParser::parse::<ExchangeClient::UserDataStreamResponse>(msg) {
                Some(Ok(exchange_message)) => {
                    if let Err(error) = account_data_tx.send(exchange_message) {
                        debug!(
                            payload = ?error.0,
                            why = "receiver dropped",
                            action = "shutting account data ws stream",
                            "failed to send account data event to Exchange receiver"
                        );
                        break;
                    }
                }
                Some(Err(err)) => {
                    if err.is_terminal() {
                        stream.cancel_running_tasks();
                        error!(
                            exchange = %exchange_id,
                            error = %err,
                            action = "Reconnecting account data web socket",
                            message = "Encounted a terminal error for account data ws"
                        );
                        break;
                    }
                }
                None => continue,
            }
        }

        // Wait a certain ms before trying to reconnect
        warn!(
            exchange = %exchange_id,
            action = "attempting re-connection after backoff",
            reconnection_attempts = connection_attempt,
        );
    }
}
