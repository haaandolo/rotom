pub mod binance;
pub mod errors;
pub mod poloniex;

use async_trait::async_trait;
use hmac::Hmac;
use rotom_data::error::SocketError;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

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

    /// **Note:**
    /// Usually entails spawning an asynchronous WebSocket event loop to consume [`AccountEvent`]s
    /// from the exchange, as well as returning the HTTP client `Self`.
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

    // Run and receive responses
    async fn receive_responses(self);

    // Transfer to another wallet
    async fn wallet_transfer(
        &self,
        coin: String,
        wallet_address: String,
    ) -> Result<Self::WalletTransferResponse, SocketError>;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(rename = "execution", rename_all = "snake_case")]
pub enum ExecutionId {
    Poloniex,
    Binance,
}
