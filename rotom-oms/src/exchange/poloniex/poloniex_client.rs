use async_trait::async_trait;
use futures::StreamExt;
use rotom_data::{
    error::SocketError,
    protocols::{
        http::{client::RestClient, http_parser::StandardHttpParser},
        ws::{connect, WsRead},
    },
};
use serde_json::Value;

use crate::{
    exchange::{ExecutionClient2, ExecutionId},
    portfolio::OrderEvent,
};

use super::{request_builder::PoloniexRequestBuilder, requests::{cancel_order::PoloniexCancelOrder, new_order::PoloniexNewOrder}};

/*----- */
// Convinent types
/*----- */
type PoloniexRestClient = RestClient<StandardHttpParser, PoloniexRequestBuilder>;
const POLONIEX_BASE_URL: &str = "https://api.poloniex.com";
const POLONIEX_USER_DATA_WS: &str = "wss://stream.binance.com:9443/ws/"; // TODO

#[derive(Debug)]
pub struct PoloniexExecution {
    // pub user_data_ws: WsRead,
    pub http_client: PoloniexRestClient,
}

#[async_trait]
impl ExecutionClient2 for PoloniexExecution {
    const CLIENT: ExecutionId = ExecutionId::Poloniex;
    type CancelResponse = Value;
    type CancelAllResponse = ();
    type NewOrderResponse = Value; // todo
    type WalletTransferResponse = ();

    async fn init() -> Result<Self, SocketError>
    where
        Self: Sized,
    {
        // Initalise rest client
        let http_client: RestClient<StandardHttpParser, PoloniexRequestBuilder> =
            RestClient::new(POLONIEX_BASE_URL, StandardHttpParser, PoloniexRequestBuilder);

        // Spin up listening ws
        // let ws = connect(POLONIEX_USER_DATA_WS).await?;
        // let (_, user_data_ws) = ws.split();

        Ok(PoloniexExecution {
            // user_data_ws,
            http_client,
        })
    }

    // Open order for single asset
    async fn open_order(
        &self,
        open_request: OrderEvent,
    ) -> Result<Self::NewOrderResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexNewOrder::new(&open_request))
            .await?;
        Ok(response.0)
    }

    // Cancel order for a single asset
    async fn cancel_order(
        &self,
        order_id: String,
        _: String,
    ) -> Result<Self::CancelResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexCancelOrder::new(order_id))
            .await?;
        Ok(response.0)
    }

    // Cancel all orders for a single asset
    async fn cancel_order_all(
        &self,
        symbol: String,
    ) -> Result<Self::CancelAllResponse, SocketError> {
        unimplemented!()
    }

    // Run and receive responses
    async fn receive_responses(self) {
        unimplemented!()
    }

    // Transfer to another wallet
    async fn wallet_transfer(
        &self,
        coin: String,
        wallet_address: String,
    ) -> Result<Self::WalletTransferResponse, SocketError> {
        unimplemented!()
    }
}
