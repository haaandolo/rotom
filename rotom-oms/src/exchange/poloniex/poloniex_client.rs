use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use rotom_data::{
    error::SocketError,
    exchange::{poloniex::PoloniexSpotPublicData, PublicStreamConnector},
    protocols::{
        http::{client::RestClient, http_parser::StandardHttpParser},
        ws::{
            connect, schedule_pings_to_exchange,
            ws_parser::{StreamParser, WebSocketParser},
            WsMessage,
        },
    },
    shared::subscription_models::ExchangeId,
};
use serde::Deserialize;

use crate::{
    exchange::{AccountDataWebsocket, ExecutionClient},
    model::{
        account_response::AccountBalance,
        execution_request::{CancelOrder, OpenOrder, WalletTransfer},
    },
};

use super::{
    request_builder::PoloniexRequestBuilder,
    requests::{
        account_data::PoloniexAccountEvents,
        balance::PoloniexBalance,
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
pub const POLONIEX_BASE_URL: &str = "https://api.poloniex.com";
const POLONIEX_USER_DATA_WS: &str = "wss://ws.poloniex.com/ws/private";

#[derive(Debug)]
pub struct PoloniexExecution {
    pub http_client: PoloniexRestClient,
}

#[async_trait]
impl ExecutionClient for PoloniexExecution {
    const CLIENT: ExchangeId = ExchangeId::PoloniexSpot;

    type PublicData = PoloniexSpotPublicData;
    type CancelResponse = Vec<PoloniexCancelOrderResponse>;
    type CancelAllResponse = Vec<PoloniexCancelOrderResponse>;
    type NewOrderResponse = PoloniexNewOrderResponse;
    type WalletTransferResponse = PoloniexWalletTransferResponse;
    type AccountDataStreamResponse = PoloniexAccountEvents;

    async fn init() -> Result<AccountDataWebsocket, SocketError> {
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
        if let Some(ping_interval) = PoloniexSpotPublicData::ping_interval() {
            let ping_handler =
                tokio::spawn(schedule_pings_to_exchange(user_data_write, ping_interval));
            tasks.push(ping_handler)
        }

        Ok(AccountDataWebsocket {
            user_data_ws,
            tasks: Some(tasks),
        })
    }

    fn new() -> Self {
        let http_client = PoloniexRestClient::new(
            POLONIEX_BASE_URL,
            StandardHttpParser,
            PoloniexRequestBuilder,
        );

        PoloniexExecution { http_client }
    }

    async fn open_order(
        &self,
        open_request: OpenOrder,
    ) -> Result<Self::NewOrderResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexNewOrder::new(open_request)?)
            .await?;
        Ok(response.0)
    }

    async fn cancel_order(
        &self,
        cancel_request: CancelOrder,
    ) -> Result<Self::CancelResponse, SocketError> {
        let response = self
            .http_client
            .execute(PoloniexCancelOrder::new(cancel_request.client_order_id.0))
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

    async fn get_balances() -> Result<Vec<AccountBalance>, SocketError> {
        let http_client = PoloniexRestClient::new(
            POLONIEX_BASE_URL,
            StandardHttpParser,
            PoloniexRequestBuilder,
        );

        let response = http_client.execute(PoloniexBalance).await?;
        let account_data: Vec<AccountBalance> = response.0.into();
        Ok(account_data)
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
