use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use rotom_data::{
    error::SocketError,
    exchange::{poloniex::PoloniexSpot, Connector},
    protocols::{
        http::{client::RestClient, http_parser::StandardHttpParser},
        ws::{
            connect, schedule_pings_to_exchange,
            ws_parser::{StreamParser, WebSocketParser},
            WsMessage,
        },
    },
};
use serde::Deserialize;

use crate::{
    exchange::{
        poloniex::requests::account_data::PoloniexAccountEvents, ExecutionClient2, ExecutionId,
        UserDataStream,
    },
    model::order::{Order, RequestOpen},
};

use super::{
    request_builder::PoloniexRequestBuilder,
    requests::{
        balance::{PoloniexBalance, PoloniexBalanceResponse},
        cancel_order::{PoloniexCancelAllOrder, PoloniexCancelOrder, PoloniexCancelOrderResponse},
        new_order::{PoloniexNewOrder, PoloniexNewOrderResponse},
        wallet_transfer::{PoloniexWalletTransfer, PoloniexWalletTransferResponse},
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
    pub http_client: PoloniexRestClient,
}

#[async_trait]
impl ExecutionClient2 for PoloniexExecution {
    const CLIENT: ExecutionId = ExecutionId::Poloniex;

    type CancelResponse = Vec<PoloniexCancelOrderResponse>;
    type CancelAllResponse = Vec<PoloniexCancelOrderResponse>;
    type NewOrderResponse = PoloniexNewOrderResponse;
    type WalletTransferResponse = PoloniexWalletTransferResponse;
    type AccountDataStreamResponse = PoloniexAccountEvents;

    async fn create_account_data_ws() -> Result<UserDataStream, SocketError> {
        // Spin up listening ws
        let ws = connect(POLONIEX_USER_DATA_WS).await?;
        let (mut user_data_write, mut user_data_ws) = ws.split();

        // Send auth to initialise ws
        let _ = user_data_write
            .send(WsMessage::text(
                serde_json::to_string(&PoloniexWsAuth::new()).unwrap(), // todo
            ))
            .await;

        // Request to subscribe to order & balance messages must be sent after auth.
        // Sometimes these requests get recognised before the auth request, hence,
        // we have to do this ugly loop. Here if we can deserialise the the auth request
        // message we know the auth worked because our PoloniexWsUserDataValidation only
        // take three message types. If successful we then send the order and balance
        // requests.
        let expected_responses: usize = 3;
        let mut success_responses: usize = 0;
        let mut sent_balance_and_order_request = false;

        loop {
            if success_responses == expected_responses {
                break;
            }

            if let Some(auth_message) = user_data_ws.next().await {
                match WebSocketParser::parse::<PoloniexWsUserDataValidation>(auth_message) {
                    // If deserialisation is sucessful send order and balance subscription request
                    Some(Ok(_)) => {
                        if !sent_balance_and_order_request {
                            // Subscribe to orders channel
                            let _ = user_data_write
                                .send(WsMessage::text(
                                    serde_json::to_string(&PoloniexWsAuthOrderRequest::new())
                                        .unwrap(), // todo
                                ))
                                .await;

                            // Subscribe to balance channel
                            let _ = user_data_write
                                .send(WsMessage::text(
                                    serde_json::to_string(&PoloniexWsAuthBalanceRequest::new())
                                        .unwrap(), // todo
                                ))
                                .await;

                            sent_balance_and_order_request = true;
                        }
                        success_responses += 1;
                    }
                    Some(Err(_)) => return Err(SocketError::PrivateDataWsSub),
                    None => continue,
                };
            }
        }

        // Handle custom ping
        let mut tasks = Vec::new();
        if let Some(ping_interval) = PoloniexSpot::ping_interval() {
            let ping_handler =
                tokio::spawn(schedule_pings_to_exchange(user_data_write, ping_interval));
            tasks.push(ping_handler)
        }

        Ok(UserDataStream {
            user_data_ws,
            tasks: Some(tasks),
        })
    }

    fn create_http_client() -> Result<Self, SocketError> {
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
        open_request: Order<RequestOpen>,
    ) -> Result<Self::NewOrderResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexNewOrder::new(&open_request)?)
            .await?;
        Ok(response.0)
    }

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

    async fn wallet_transfer(
        &self,
        coin: String,
        wallet_address: String,
        network: Option<String>,
        amount: f64,
    ) -> Result<Self::WalletTransferResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexWalletTransfer::new(
                coin,
                network,
                amount,
                wallet_address,
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
