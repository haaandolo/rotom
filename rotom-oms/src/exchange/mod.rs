pub mod binance;
pub mod errors;
pub mod poloniex;

use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use binance::binance_client::BinanceExecution;
use futures::StreamExt;
use hmac::Hmac;
use poloniex::poloniex_client::PoloniexExecution;
use rotom_data::{
    error::SocketError,
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
    account_data::AccountData,
    order::{CancelOrder, OpenOrder, WalletTransfer},
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

    type CancelResponse;
    type CancelAllResponse;
    type NewOrderResponse;
    type WalletTransferResponse;
    type AccountDataStreamResponse: Send + for<'de> Deserialize<'de> + Debug + Into<AccountData>;

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
}

/*----- */
// Account User Data Auto Reconnect
/*----- */
pub async fn consume_account_data_stream<Exchange>(
    account_data_tx: mpsc::UnboundedSender<AccountData>,
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
// Account Data Stream - Send to corresponding trader
/*----- */
pub async fn send_account_data_to_traders(
    trader_tx: HashMap<String, mpsc::Sender<AccountData>>,
    mut account_data_stream: mpsc::UnboundedReceiver<AccountData>,
) {
    while let Some(message) = account_data_stream.recv().await {
        match message {
            AccountData::Order(order) => {
                if let Some(trader_tx) = trader_tx.get(&order.asset) {
                    let _ = trader_tx.send(AccountData::Order(order)).await;
                }
            }
            AccountData::BalanceVec(balances) => {
                for balance in balances.into_iter() {
                    if let Some(trader_tx) = trader_tx.get(&balance.asset) {
                        let _ = trader_tx.send(AccountData::Balance(balance)).await;
                    }
                }
            }
            AccountData::BalanceDelta(balance_delta) => {
                if let Some(trader_tx) = trader_tx.get(&balance_delta.asset) {
                    let _ = trader_tx
                        .send(AccountData::BalanceDelta(balance_delta))
                        .await;
                }
            }
            AccountData::Balance(balance) => {
                if let Some(trader_tx) = trader_tx.get(&balance.asset) {
                    let _ = trader_tx.send(AccountData::Balance(balance)).await;
                }
            }
        }
    }
}

/*----- */
// Account Data Stream - combine streams together
/*----- */
pub async fn combine_account_data_stream(
    exchange_ids: Vec<ExchangeId>,
    trader_order_updater: HashMap<String, mpsc::Sender<AccountData>>,
) {
    // Init account data channels and combine
    let (account_data_tx, account_data_rx) = mpsc::unbounded_channel();
    for exchange in exchange_ids.into_iter() {
        match exchange {
            ExchangeId::BinanceSpot => {
                tokio::spawn(consume_account_data_stream::<BinanceExecution>(
                    account_data_tx.clone(),
                ));
            }
            ExchangeId::PoloniexSpot => {
                tokio::spawn(consume_account_data_stream::<PoloniexExecution>(
                    account_data_tx.clone(),
                ));
            }
        }
    }

    // Send order updates to corresponding trader pair for exchange
    tokio::spawn(send_account_data_to_traders(
        trader_order_updater,
        account_data_rx,
    ));
}
