use std::fmt::Debug;

use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::{time::sleep, time::Duration};
use tracing::{error, info, warn};

use crate::data::error::SocketError;
use crate::data::protocols::ws::{create_websocket, WsMessage};
use crate::data::transformer::ExchangeTransformer;
use crate::data::{
    exchange::{Connector, StreamSelector},
    model::{event::MarketEvent, subs::Subscription, SubKind},
};

pub const START_RECONNECTION_BACKOFF_MS: u64 = 125;

pub async fn consume<Exchange, StreamKind>(
    exchange_sub: Vec<Subscription<Exchange, StreamKind>>,
    exchange_tx: UnboundedSender<MarketEvent<StreamKind::Event>>,
) -> SocketError
where
    StreamKind: SubKind,
    Exchange: Connector + Send + StreamSelector<Exchange, StreamKind> + Debug,
    Exchange::StreamTransformer: ExchangeTransformer<Exchange::Stream, StreamKind>,
{
    let exchange_id = Exchange::ID;

    info!(
        exchange = %exchange_id,
        ?exchange_sub,
        action = "Attempting to subscribe to websocket"
    );

    let mut connection_attempt: u32 = 0;
    let mut backoff_ms: u64 = START_RECONNECTION_BACKOFF_MS;

    // TODO: I DONT LIKE THIS - MAKE IT CLEANER
    let subs = exchange_sub
        .into_iter()
        .map(|s| s.instrument)
        .collect::<Vec<_>>();

    loop {
        connection_attempt += 1;
        backoff_ms *= 2;

        // Attempt to connect to the stream
        let mut stream = match create_websocket::<Exchange, StreamKind>(&subs).await {
            Ok(stream) => {
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

        // Validate subscriptions
        if let Some(Ok(WsMessage::Text(message))) = &stream.ws_read.next().await {
            let subscription_sucess = Exchange::validate_subscription(message.to_owned(), &subs);

            if !subscription_sucess {
                break SocketError::Subscribe(String::from("Subscription failed"));
            }
        };

        // Read from stream and send via channel, but if error occurs, attempt reconnection
        while let Some(market_event) = stream.next().await {
            match market_event {
                Ok(market_event) => {
                    exchange_tx
                        .send(market_event)
                        .expect("failed to send message");
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

                // If error is non-terminal, just continue
                Err(error) => {
                    warn!(
                        exchange = %exchange_id,
                        error = %error,
                        action = "Continuing...",
                        message = "Encountered a non-terminal error",
                    );
                    continue;
                }
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
