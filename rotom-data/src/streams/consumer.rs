use std::fmt::Debug;

use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::{time::sleep, time::Duration};
use tracing::{debug, error, info, warn};

use crate::error::{SocketError, ACCEPTABLE_DE_ERROR_MESSAGES};
use crate::exchange::Identifier;
use crate::protocols::ws::WebSocketClient;
use crate::shared::subscription_models::Subscription;
use crate::transformer::ExchangeTransformer;
use crate::{
    exchange::{PublicStreamConnector, StreamSelector},
    model::{market_event::MarketEvent, SubKind},
};

pub const START_RECONNECTION_BACKOFF_MS: u64 = 125;

pub async fn consume<Exchange, StreamKind>(
    exchange_sub: Vec<Subscription<Exchange, StreamKind>>,
    exchange_tx: UnboundedSender<MarketEvent<StreamKind::Event>>,
) -> SocketError
where
    StreamKind: SubKind,
    Exchange:
        PublicStreamConnector + Send + StreamSelector<Exchange, StreamKind> + Debug + Clone + Sync,
    Exchange::StreamTransformer: ExchangeTransformer<Exchange, Exchange::Stream, StreamKind>,
    Subscription<Exchange, StreamKind>:
        Identifier<Exchange::Channel> + Identifier<Exchange::Market> + Debug,
{
    let exchange_id = Exchange::ID;
    let mut connection_attempt: u32 = 0;
    let mut backoff_ms: u64 = START_RECONNECTION_BACKOFF_MS;

    info!(
        exchange = %exchange_id,
        ?exchange_sub,
        action = "Attempting to subscribe to websocket"
    );

    loop {
        connection_attempt += 1;
        backoff_ms *= 2;

        // Attempt to connect to the stream
        let mut stream = match WebSocketClient::init(&exchange_sub).await {
            Ok(stream) => {
                // Comment-out below if you want reconnection attempt to at each loop iteration
                connection_attempt = 0;
                backoff_ms = START_RECONNECTION_BACKOFF_MS;
                stream
            }
            Err(error) => {
                if connection_attempt == 1 {
                    return SocketError::Subscribe(format!(
                        "Subscription failed on first attempt: {}",
                        error
                    ));
                } else {
                    continue;
                }
            }
        };

        // Read from stream and send via channel, but if error occurs, attempt reconnection
        while let Some(market_event) = stream.next().await {
            match market_event {
                Ok(market_event) => {
                    if let Err(error) = exchange_tx.send(market_event) {
                        debug!(
                            payload = ?error.0,
                            why = "receiver dropped",
                            action = "shutting down Stream",
                            "failed to send Event<MarketData> to Exchange receiver"
                        );
                        break;
                    }
                }

                // If error is terminal e.g. invalid sequence, then break
                Err(error) if error.is_terminal() => {
                    stream.cancel_running_tasks();
                    error!(
                        exchange = %exchange_id,
                        error = %error,
                        action = "Reconnecting web socket",
                        message = "Encounted a terminal error"
                    );
                    break;
                }

                // If error is non-terminal, just continue or log and continue
                Err(error) => match error {
                    // This error is harmless so dont log and continue
                    SocketError::TransformerNone => continue,
                    // Some de errors are harmless so we dont want to log e.g poloniex exchange pings
                    SocketError::Deserialise { error, payload } => {
                        if ACCEPTABLE_DE_ERROR_MESSAGES.contains(&payload.as_str()) {
                            continue;
                        } else {
                            warn!(
                                exchange = %exchange_id,
                                error = %error,
                                action = "Continuing...",
                                message = "Encountered a non-terminal error",
                            );
                            continue;
                        }
                    }
                    // However other errors need logging
                    _ => {
                        warn!(
                            exchange = %exchange_id,
                            error = %error,
                            action = "Continuing...",
                            message = "Encountered a non-terminal error",
                        );
                        continue;
                    }
                },
            }
        }

        // Wait a certain ms before trying to reconnect
        warn!(
            exchange = %exchange_id,
            action = "attempting re-connection after backoff",
            reconnection_attempts = connection_attempt,
        );
        sleep(Duration::from_millis(backoff_ms)).await;
    }
}
