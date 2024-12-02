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

use crate::model::{account_data::AccountData, order::OrderEvent};

/*----- */
// Convenient types
/*----- */
type HmacSha256 = Hmac<Sha256>;

/*----- */
// Private Data Ws Stream
/*----- */
// todo: change to accountDataStream
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
// Account Data Trait
/*----- */
#[async_trait]
pub trait AccountDataStream {
    const CLIENT: ExecutionId;
    type AccountDataStreamResponse: Send + for<'de> Deserialize<'de> + Debug + Into<AccountData>;

    async fn init() -> Result<UserDataStream, SocketError>;
}

/*----- */
// Execution Client Trait
/*----- */
#[async_trait]
pub trait ExecutionClient {
    const CLIENT: ExecutionId;

    type CancelResponse;
    type CancelAllResponse;
    type NewOrderResponse;
    type WalletTransferResponse;

    // Init exchange executor
    fn new() -> Result<Self, SocketError>
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
// Account User Data Auto Reconnect
/*----- */
pub async fn consume_account_data_stream<ExchangeAccountDataStream>(
    account_data_tx: mpsc::UnboundedSender<AccountData>,
) -> Result<(), SocketError>
where
    ExchangeAccountDataStream: AccountDataStream,
{
    let exchange_id = ExchangeAccountDataStream::CLIENT;
    let mut connection_attempt: u32 = 0;
    let mut _backoff_ms: u64 = 125;

    info!(
        exchange = %exchange_id,
        action = "Connecting to private user websocket stream"
    );

    loop {
        let mut stream = ExchangeAccountDataStream::init().await?;
        connection_attempt += 1;
        _backoff_ms *= 2;

        while let Some(msg) = stream.user_data_ws.next().await {
            match WebSocketParser::parse::<ExchangeAccountDataStream::AccountDataStreamResponse>(
                msg,
            ) {
                Some(Ok(exchange_message)) => {
                    if let Err(error) = account_data_tx.send(exchange_message.into()) {
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

/*----- */
// Account Data Stream - Combine Streams
/*----- */
pub async fn combine_account_data_streams<StreamOne, StreamTwo>(
) -> mpsc::UnboundedReceiver<AccountData>
where
    StreamOne: AccountDataStream + 'static,
    StreamTwo: AccountDataStream + 'static,
{
    // Convert first exchange ws to channel
    let (exchange_one_tx, mut exchange_one_rx) = mpsc::unbounded_channel();
    tokio::spawn(consume_account_data_stream::<StreamOne>(exchange_one_tx));

    // Convert second exchange ws to channel
    let (exchange_two_tx, mut exchange_two_rx) = mpsc::unbounded_channel();
    tokio::spawn(consume_account_data_stream::<StreamTwo>(exchange_two_tx));

    // Combine channels into one
    let (combined_tx, combined_rx) = mpsc::unbounded_channel();
    let combined_tx_cloned = combined_tx.clone();
    tokio::spawn(async move {
        while let Some(message) = exchange_one_rx.recv().await {
            let _ = combined_tx_cloned.send(message);
        }
    });

    tokio::spawn(async move {
        while let Some(message) = exchange_two_rx.recv().await {
            let _ = combined_tx.send(message);
        }
    });

    combined_rx
}
