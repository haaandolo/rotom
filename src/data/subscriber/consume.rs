use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::{time::sleep, time::Duration};

use crate::data::protocols::ws::{WebSocketClient, WsMessage};
use crate::{
    data::{
        exchange::{Connector, StreamSelector},
        models::{event::MarketEvent, subs::Subscription, SubKind},
        transformer::Transformer,
    },
    error::SocketError,
};

pub const START_RECONNECTION_BACKOFF_MS: u64 = 125;

pub async fn consume<Exchange, StreamKind>(
    exchange_sub: Vec<Subscription<Exchange, StreamKind>>,
    exchange_tx: UnboundedSender<MarketEvent<StreamKind::Event>>,
) -> SocketError
where
    Exchange: Connector + Send + StreamSelector<Exchange, StreamKind>,
    StreamKind: SubKind,
    MarketEvent<StreamKind::Event>: From<<Exchange::StreamTransformer as Transformer>::Output>,
{
    let mut _connection_attempt: u32 = 0;
    let mut _backoff_ms: u64 = START_RECONNECTION_BACKOFF_MS;

    // I DONT LIKE THIS - MAKE IT CLEANER
    let subs = exchange_sub
        .into_iter()
        .map(|s| s.instrument)
        .collect::<Vec<_>>();

    loop {
        _connection_attempt += 1;
        _backoff_ms *= 2;

        // Attempt to connect to the stream
        let mut stream =
            match WebSocketClient::<Exchange, StreamKind>::create_websocket(&subs).await {
                Ok(stream) => {
                    _connection_attempt = 0;
                    _backoff_ms = START_RECONNECTION_BACKOFF_MS;
                    stream
                }
                Err(error) => {
                    if _connection_attempt == 1 {
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
                        .send(market_event.into())
                        .expect("failed to send message");
                }
                Err(_error) => {
                    // SHOULD CHANGE THIS TO SEE WHAT ERRORS ACTUAL MEANS INSTEAD OF BREAKING
                    stream.cancel_running_tasks();
                    break;
                } // ADD Error for if sequence is broken then have to restart
            }
        }

        // Wait a certain ms before trying to reconnect
        sleep(Duration::from_millis(_backoff_ms)).await;
    }
}
