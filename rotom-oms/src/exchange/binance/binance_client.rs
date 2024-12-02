use async_trait::async_trait;
use rotom_data::error::SocketError;
use rotom_data::protocols::http::client::RestClient;
use rotom_data::protocols::http::http_parser::StandardHttpParser;

use crate::exchange::binance::requests::cancel_order::BinanceCancelOrder;
use crate::exchange::binance::requests::new_order::BinanceNewOrder;
use crate::exchange::binance::requests::wallet_transfer::BinanceWalletTransfer;
use crate::exchange::ExecutionClient;
use crate::exchange::ExecutionId;
use crate::model::order::OrderEvent;

use super::request_builder::BinanceRequestBuilder;
use super::requests::balance::BinanceBalance;
use super::requests::balance::BinanceBalanceResponse;
use super::requests::cancel_order::BinanceCancelAllOrder;
use super::requests::cancel_order::BinanceCancelOrderResponse;
use super::requests::new_order::BinanceNewOrderResponses;
use super::requests::wallet_transfer::BinanceWalletTransferResponse;

/*----- */
// Convinent types
/*----- */
pub type BinanceRestClient = RestClient<StandardHttpParser, BinanceRequestBuilder>;
pub const BINANCE_BASE_URL: &str = "https://api.binance.com";

#[derive(Debug)]
pub struct BinanceExecution {
    pub http_client: BinanceRestClient,
}

#[async_trait]
impl ExecutionClient for BinanceExecution {
    const CLIENT: ExecutionId = ExecutionId::Binance;

    type CancelResponse = BinanceCancelOrderResponse;
    type CancelAllResponse = Vec<BinanceCancelOrderResponse>;
    type NewOrderResponse = BinanceNewOrderResponses;
    type WalletTransferResponse = BinanceWalletTransferResponse;

    fn new() -> Result<Self, SocketError> {
        let http_client =
            BinanceRestClient::new(BINANCE_BASE_URL, StandardHttpParser, BinanceRequestBuilder);
        Ok(BinanceExecution { http_client })
    }

    async fn open_order(
        &self,
        open_requests: OrderEvent,
    ) -> Result<Self::NewOrderResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceNewOrder::new(&open_requests)?)
            .await?;
        Ok(response.0)
    }

    async fn cancel_order(
        &self,
        orig_client_order_id: String,
        symbol: String,
    ) -> Result<Self::CancelResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceCancelOrder::new(orig_client_order_id, symbol)?)
            .await?;
        Ok(response.0)
    }

    async fn cancel_order_all(
        &self,
        symbol: String,
    ) -> Result<Self::CancelAllResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceCancelAllOrder::new(symbol)?)
            .await?;
        Ok(response.0)
    }

    async fn wallet_transfer(
        &self,
        coin: String,
        wallet_address: String,
        network: Option<String>,
        amount: f64,
    ) -> Result<Self::WalletTransferResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceWalletTransfer::new(
                coin,
                wallet_address,
                network,
                amount,
            )?)
            .await?;
        Ok(response.0)
    }
}

/*----- */
// Binance Private Data
/*----- */
#[derive(Debug)]
pub struct BinancePrivateData {
    pub http_client: BinanceRestClient,
}

impl Default for BinancePrivateData {
    fn default() -> Self {
        BinancePrivateData::new()
    }
}

impl BinancePrivateData {
    pub fn new() -> Self {
        let http_client =
            RestClient::new(BINANCE_BASE_URL, StandardHttpParser, BinanceRequestBuilder);
        Self { http_client }
    }

    #[inline]
    pub async fn get_balance_all(&self) -> Result<BinanceBalanceResponse, SocketError> {
        let response = self.http_client.execute(BinanceBalance::new()?).await?;
        Ok(response.0)
    }
}
