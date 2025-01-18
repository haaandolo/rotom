pub mod binance;
pub mod errors;
pub mod poloniex;

use std::fmt::Debug;

use async_trait::async_trait;
use futures::StreamExt;
use hmac::Hmac;
use rotom_data::{
    error::SocketError,
    exchange::{PublicHttpConnector, PublicStreamConnector},
    protocols::ws::{
        ws_parser::{StreamParser, WebSocketParser},
        JoinHandle, WsRead,
    },
    shared::subscription_models::ExchangeId,
};
use serde::Deserialize;
use sha2::Sha256;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::model::{
    execution_request::{CancelOrder, OpenOrder, WalletTransfer},
    execution_response::{AccountBalance, ExecutionResponse},
};

/*----- */
// Convenient types
/*----- */
type HmacSha256 = Hmac<Sha256>;

/*----- */
// Private Data Ws Stream
/*----- */
pub struct AccountDataWebsocket {
    pub user_data_ws: WsRead,
    pub tasks: Option<Vec<JoinHandle>>,
}

impl AccountDataWebsocket {
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
pub trait ExecutionClient {
    const CLIENT: ExchangeId;

    type PublicData: PublicStreamConnector + PublicHttpConnector;
    type CancelResponse: Send + Debug;
    type CancelAllResponse: Send + Debug;
    type NewOrderResponse: Send + Debug;
    type WalletTransferResponse: Send + Debug;
    type AccountDataStreamResponse: Send
        + for<'de> Deserialize<'de>
        + Debug
        + Into<ExecutionResponse>;

    // Initialise a account data stream
    async fn init() -> Result<AccountDataWebsocket, SocketError>;

    // Init exchange executor
    fn new() -> Self
    where
        Self: Sized;

    // Open order for single asset
    async fn open_order(
        &self,
        open_request: OpenOrder,
    ) -> Result<Self::NewOrderResponse, SocketError>;

    // Cancel order for a single asset
    async fn cancel_order(
        &self,
        cancel_request: CancelOrder,
    ) -> Result<Self::CancelResponse, SocketError>;

    // Cancel all orders for a single asset
    async fn cancel_order_all(
        &self,
        cancel_request: CancelOrder,
    ) -> Result<Self::CancelAllResponse, SocketError>;

    // Transfer to another wallet
    async fn wallet_transfer(
        &self,
        wallet_transfer_request: WalletTransfer,
    ) -> Result<Self::WalletTransferResponse, SocketError>;

    // Get all balance figure on a exchange
    async fn get_balances() -> Result<Vec<AccountBalance>, SocketError>;
}

/*----- */
// Account User Data Auto Reconnect
/*----- */
pub async fn consume_account_data_stream<Exchange>(
    account_data_tx: mpsc::UnboundedSender<ExecutionResponse>,
) -> Result<(), SocketError>
where
    Exchange: ExecutionClient,
{
    let exchange_id = Exchange::CLIENT;
    let mut connection_attempt: u32 = 0;
    let mut _backoff_ms: u64 = 125;

    info!(
        exchange = %exchange_id,
        action = "Connecting to private user websocket stream"
    );

    loop {
        let mut stream = Exchange::init().await?;
        connection_attempt += 1;
        _backoff_ms *= 2;

        while let Some(msg) = stream.user_data_ws.next().await {
            match WebSocketParser::parse::<Exchange::AccountDataStreamResponse>(msg) {
                Some(Ok(exchange_message)) => {
                    println!("### Parsed  ###");
                    println!("{:#?}", exchange_message);
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
