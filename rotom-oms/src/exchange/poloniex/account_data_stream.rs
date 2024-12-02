use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use rotom_data::{
    error::SocketError,
    exchange::{poloniex::PoloniexSpot, Connector},
    protocols::ws::{
        connect, schedule_pings_to_exchange,
        ws_parser::{StreamParser, WebSocketParser},
        WsMessage,
    },
};

use crate::exchange::{AccountDataStream, AccountDataWebsocket, ExecutionId};

use super::{
    poloniex_client::PoloniexWsUserDataValidation,
    requests::{
        account_data::PoloniexAccountEvents,
        ws_auth::{PoloniexWsAuth, PoloniexWsAuthBalanceRequest, PoloniexWsAuthOrderRequest},
    },
};

const POLONIEX_USER_DATA_WS: &str = "wss://ws.poloniex.com/ws/private";

#[derive(Debug)]
pub struct PoloniexAccountDataStream;

#[async_trait]
impl AccountDataStream for PoloniexAccountDataStream {
    const CLIENT: ExecutionId = ExecutionId::Poloniex;
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
        if let Some(ping_interval) = PoloniexSpot::ping_interval() {
            let ping_handler =
                tokio::spawn(schedule_pings_to_exchange(user_data_write, ping_interval));
            tasks.push(ping_handler)
        }

        Ok(AccountDataWebsocket {
            user_data_ws,
            tasks: Some(tasks),
        })
    }
}
