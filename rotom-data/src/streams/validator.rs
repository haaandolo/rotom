use std::fmt::Debug;

use async_trait::async_trait;
use futures::StreamExt;
use tracing::debug;

use crate::{
    error::SocketError,
    exchange::PublicStreamConnector,
    protocols::ws::{
        ws_parser::{StreamParser, WebSocketParser},
        WsRead,
    },
    shared::subscription_models::ExchangeSubscription,
};
/*----- */
// Validator
/*----- */
pub trait Validator {
    /// Check if `Self` is valid for some use case.
    fn validate(self) -> Result<Self, SocketError>
    where
        Self: Sized;
}

/*----- */
// Subscription validator
/*----- */
#[async_trait]
pub trait SubscriptionValidator {
    type Parser: StreamParser;

    async fn validate<Exchange>(
        subscriptions: &[ExchangeSubscription<Exchange, Exchange::Channel, Exchange::Market>],
        websocket: WsRead,
    ) -> Result<WsRead, SocketError>
    where
        Exchange: PublicStreamConnector + Send + Sync,
        Exchange::SubscriptionResponse: Validator + Send + Debug;
}

pub struct WebSocketValidator;

#[async_trait]
impl SubscriptionValidator for WebSocketValidator {
    type Parser = WebSocketParser;

    async fn validate<Exchange>(
        subscriptions: &[ExchangeSubscription<Exchange, Exchange::Channel, Exchange::Market>],
        mut websocket: WsRead,
    ) -> Result<WsRead, SocketError>
    where
        Exchange: PublicStreamConnector + Send + Sync,
        Exchange::SubscriptionResponse: Validator + Send + Debug,
    {
        let exchange_id = Exchange::ID;
        let timeout = Exchange::subscription_validation_timeout();
        let expected_responses = Exchange::expected_responses(subscriptions);
        let mut success_responses: usize = 0;
        loop {
            if success_responses == expected_responses {
                break Ok(websocket);
            }

            tokio::select! {
                // If timeout reached, return SubscribeError
                _ = tokio::time::sleep(timeout) => {
                    break Err(SocketError::Subscribe(
                        format!("subscription validation timeout reached: {:?}", timeout)
                    ))
                },
                // Parse incoming messages and determine subscription outcomes
                message = websocket.next() => {
                    let response = match message {
                        Some(response) => response,
                        None => break Err(SocketError::Subscribe("WebSocket stream terminated unexpectedly".to_string()))
                    };

                    match Self::Parser::parse::<Exchange::SubscriptionResponse>(response) {
                        Some(Ok(response)) => match response.validate() {
                            // Subscription success
                            Ok(response) => {
                                success_responses += 1;
                                debug!(
                                    exchange = %exchange_id,
                                    %success_responses,
                                    %expected_responses,
                                    payload = ?response,
                                    "received valid Ok subscription response",
                                );
                            }

                            // Subscription failure
                            Err(err) => break Err(err)
                        }
                        Some(Err(SocketError::Deserialise { error, payload })) if success_responses >= 1 => {
                            // Already active subscription payloads, so skip to next SubResponse
                            debug!(
                                exchange = %Exchange::ID,
                                ?error,
                                %success_responses,
                                %expected_responses,
                                %payload,
                                "failed to deserialise non SubResponse payload"
                            );
                            continue
                        }
                        Some(Err(SocketError::Terminated(close_frame))) => {
                            break Err(SocketError::Subscribe(
                                format!("received WebSocket CloseFrame: {close_frame}")
                            ))
                        }
                        _ => {
                            // Pings, Pongs, Frames, etc.
                            continue
                        }
                    }
                }
            }
        } //
    }
}
