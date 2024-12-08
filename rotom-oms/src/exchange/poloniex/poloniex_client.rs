use async_trait::async_trait;
use rotom_data::{
    error::SocketError,
    protocols::http::{client::RestClient, http_parser::StandardHttpParser},
};
use serde::Deserialize;

use crate::{
    exchange::{ExecutionClient, ExecutionId},
    model::order::{CancelOrder, OpenOrder, WalletTransfer},
};

use super::{
    request_builder::PoloniexRequestBuilder,
    requests::{
        balance::{PoloniexBalance, PoloniexBalanceResponse},
        cancel_order::{PoloniexCancelAllOrder, PoloniexCancelOrder, PoloniexCancelOrderResponse},
        new_order::{PoloniexNewOrder, PoloniexNewOrderResponse},
        wallet_transfer::{PoloniexWalletTransfer, PoloniexWalletTransferResponse},
    },
};

/*----- */
// Convinent types
/*----- */
type PoloniexRestClient = RestClient<StandardHttpParser, PoloniexRequestBuilder>;
const POLONIEX_BASE_URL: &str = "https://api.poloniex.com";

#[derive(Debug)]
pub struct PoloniexExecution {
    pub http_client: PoloniexRestClient,
}

#[async_trait]
impl ExecutionClient for PoloniexExecution {
    const CLIENT: ExecutionId = ExecutionId::Poloniex;

    type CancelResponse = Vec<PoloniexCancelOrderResponse>;
    type CancelAllResponse = Vec<PoloniexCancelOrderResponse>;
    type NewOrderResponse = PoloniexNewOrderResponse;
    type WalletTransferResponse = PoloniexWalletTransferResponse;

    fn new() -> Result<Self, SocketError> {
        // Initalise rest client
        let http_client = PoloniexRestClient::new(
            POLONIEX_BASE_URL,
            StandardHttpParser,
            PoloniexRequestBuilder,
        );

        Ok(PoloniexExecution { http_client })
    }

    async fn open_order(
        &self,
        open_request: OpenOrder,
    ) -> Result<Self::NewOrderResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexNewOrder::new(&open_request)?)
            .await?;
        Ok(response.0)
    }

    async fn cancel_order(
        &self,
        cancel_request: CancelOrder,
    ) -> Result<Self::CancelResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexCancelOrder::new(cancel_request.id))
            .await?;
        Ok(response.0)
    }

    async fn cancel_order_all(
        &self,
        cancel_request: CancelOrder,
    ) -> Result<Self::CancelAllResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexCancelAllOrder::new(cancel_request.symbol))
            .await?;
        Ok(response.0)
    }

    async fn wallet_transfer(
        &self,
        wallet_transfer_request: WalletTransfer,
    ) -> Result<Self::WalletTransferResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexWalletTransfer::new(
                wallet_transfer_request.coin,
                wallet_transfer_request.network,
                wallet_transfer_request.amount,
                wallet_transfer_request.wallet_address,
            ))
            .await?;
        Ok(response.0)
    }
}

/*----- */
// Poloniex Private Data
/*----- */
#[derive(Debug)]
pub struct PoloniexPrivateData {
    pub http_client: PoloniexRestClient,
}

impl Default for PoloniexPrivateData {
    fn default() -> Self {
        PoloniexPrivateData::new()
    }
}

impl PoloniexPrivateData {
    pub fn new() -> Self {
        let http_client = RestClient::new(
            POLONIEX_BASE_URL,
            StandardHttpParser,
            PoloniexRequestBuilder,
        );
        Self { http_client }
    }

    #[inline]
    pub async fn get_balance_all(&self) -> Result<PoloniexBalanceResponse, SocketError> {
        let response = self.http_client.execute(PoloniexBalance).await?;
        Ok(response.0)
    }
}

/*----- */
// Poloniex ws auth responses
/*----- */
#[derive(Debug, Deserialize)]
pub struct PoloniexWsResponseAuthMessage {
    pub data: PoloniexWsResponseAuthMessageData,
    pub channel: String, // can be smolstr
}

#[derive(Debug, Deserialize)]
pub struct PoloniexWsResponseAuthMessageData {
    pub success: bool,
    pub message: Option<String>, // can be smolstr
    pub ts: u64,
}

#[derive(Debug, Deserialize)]
pub struct PoloniexWsOrderResponse {
    pub channel: String,      // can be smolstr
    pub event: String,        // can be smolstr
    pub symbols: Vec<String>, // can be smolstr
}

#[derive(Debug, Deserialize)]
pub struct PoloniexWsBalanceResponse {
    pub channel: String, // can be smolstr
    pub event: String,   // can be smolstr
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PoloniexWsUserDataValidation {
    Auth(PoloniexWsResponseAuthMessage),
    Orders(PoloniexWsOrderResponse),
    Balance(PoloniexWsBalanceResponse),
}
