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
        WsRead,
    },
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::mpsc;

use crate::portfolio::OrderEvent;

/*----- */
// Convenient types
/*----- */
type HmacSha256 = Hmac<Sha256>;

/*----- */
// Execution Clinet Trait
/*----- */
#[async_trait]
pub trait ExecutionClient2 {
    const CLIENT: ExecutionId;

    type CancelResponse;
    type CancelAllResponse;
    type NewOrderResponse;
    type WalletTransferResponse;
    type UserDataStreamResponse;

    // **Note:**
    // Usually entails spawning an asynchronous WebSocket event loop to consume [`AccountEvent`]s
    // from the exchange, as well as returning the HTTP client `Self`.
    async fn init() -> Result<Self, SocketError>
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

    // // Run and receive responses
    // async fn receive_responses(
    //     self,
    // ) -> Result<mpsc::UnboundedReceiver<Self::UserDataStreamResponse>, SocketError>;

    // Transfer to another wallet
    async fn wallet_transfer(
        &self,
        coin: String,
        wallet_address: String,
        network: Option<String>,
        amount: f64,
    ) -> Result<Self::WalletTransferResponse, SocketError>;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(rename = "execution", rename_all = "snake_case")]
pub enum ExecutionId {
    Poloniex,
    Binance,
}

/*----- */
// Function to spawn and return a tx for private user ws
/*----- */
pub async fn get_user_data_read_channel<UserDataResponse>(
    mut web_socket: WsRead,
) -> Result<mpsc::UnboundedReceiver<UserDataResponse>, SocketError>
where
    UserDataResponse: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    let (tx, rx) = mpsc::unbounded_channel();
    while let Some(msg) = web_socket.next().await {
        println!("$$$$$ {:#?}", msg);
        let response = match WebSocketParser::parse::<UserDataResponse>(msg) {
            Some(Ok(exchange_message)) => exchange_message,
            Some(Err(err)) => return Err(err),
            None => continue,
        };
        let _ = tx.send(response);
    }
    Ok(rx)
}

pub async fn spawn_ws_read<UserDataResponse>(
    mut web_socket: WsRead,
    tx: mpsc::UnboundedSender<UserDataResponse>,
) -> Result<(), SocketError>
where
    UserDataResponse: for<'de> Deserialize<'de>,
{
    while let Some(msg) = web_socket.next().await {
        let response = match WebSocketParser::parse::<UserDataResponse>(msg) {
            Some(Ok(exchange_message)) => exchange_message,
            Some(Err(err)) => return Err(err),
            None => continue,
        };
        let _ = tx.send(response);
    }
    Ok(())
}
