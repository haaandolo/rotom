use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use rotom_data::{
    error::SocketError,
    exchange::{poloniex::PoloniexSpot, Connector},
    protocols::{
        http::{client::RestClient, http_parser::StandardHttpParser},
        ws::{connect, schedule_pings_to_exchange, JoinHandle, WsMessage, WsRead},
    },
};
use serde_json::Value;

use crate::{
    exchange::{ExecutionClient2, ExecutionId},
    portfolio::OrderEvent,
};

use super::{
    request_builder::PoloniexRequestBuilder,
    requests::{
        cancel_order::{PoloniexCancelAllOrder, PoloniexCancelOrder},
        new_order::PoloniexNewOrder,
        ws_auth::{PoloniexWsAuth, PoloniexWsAuthBalanceRequest, PoloniexWsAuthOrderRequest},
    },
};

/*----- */
// Convinent types
/*----- */
type PoloniexRestClient = RestClient<StandardHttpParser, PoloniexRequestBuilder>;
const POLONIEX_BASE_URL: &str = "https://api.poloniex.com";
const POLONIEX_USER_DATA_WS: &str = "wss://ws.poloniex.com/ws/private";

#[derive(Debug)]
pub struct PoloniexExecution {
    pub user_data_ws: WsRead,
    pub http_client: PoloniexRestClient,
    pub tasks: Vec<JoinHandle>,
}

#[async_trait]
impl ExecutionClient2 for PoloniexExecution {
    const CLIENT: ExecutionId = ExecutionId::Poloniex;
    type CancelResponse = Value;
    type CancelAllResponse = Value;
    type NewOrderResponse = Value; // todo
    type WalletTransferResponse = ();

    async fn init() -> Result<Self, SocketError>
    where
        Self: Sized,
    {
        // Initalise rest client
        let http_client: RestClient<StandardHttpParser, PoloniexRequestBuilder> = RestClient::new(
            POLONIEX_BASE_URL,
            StandardHttpParser,
            PoloniexRequestBuilder,
        );

        // Spin up listening ws
        let ws = connect(POLONIEX_USER_DATA_WS).await?;
        let (mut user_data_write, user_data_ws) = ws.split();

        // Send auth to initialise ws
        let _ = user_data_write
            .send(WsMessage::text(
                serde_json::to_string(&PoloniexWsAuth::new()).unwrap(), // todo
            ))
            .await;

        // Subscribe to orders channel
        let _ = user_data_write
            .send(WsMessage::text(
                serde_json::to_string(&PoloniexWsAuthOrderRequest::new()).unwrap(), // todo
            ))
            .await;

        // Subscribe to balance channel
        let _ = user_data_write
            .send(WsMessage::text(
                serde_json::to_string(&PoloniexWsAuthBalanceRequest::new()).unwrap(), // todo
            ))
            .await;

        // Handle custom ping
        let mut tasks = Vec::new();
        if let Some(ping_interval) = PoloniexSpot::ping_interval() {
            let ping_handler =
                tokio::spawn(schedule_pings_to_exchange(user_data_write, ping_interval));
            tasks.push(ping_handler)
        }

        Ok(PoloniexExecution {
            user_data_ws,
            http_client,
            tasks,
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
        let response = self
            .http_client
            .execute(PoloniexCancelAllOrder::new(symbol))
            .await?;
        Ok(response.0)
    }

    // Run and receive responses
    async fn receive_responses(mut self) {
        while let Some(msg) = self.user_data_ws.next().await {
            println!(">>> {:#?}", msg);
        }
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

/*
>>> Ok(
    Text(
        "{\"channel\":\"auth\",\"data\":{\"success\":true,\"ts\":1729907814124}}",
    ),
)
>>> Ok(
    Text(
        "{\"event\":\"subscribe\",\"channel\":\"orders\",\"symbols\":[\"ALL\"]}",
    ),
)
>>> Ok(
    Text(
        "{\"event\":\"subscribe\",\"channel\":\"balances\"}",
    ),
)
*/
