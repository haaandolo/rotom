pub mod ws_client;

use std::fmt::Debug;

use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{sleep, Duration},
};
use tokio_tungstenite::tungstenite::{
    error::ProtocolError,
    protocol::{frame::Frame, CloseFrame},
};
use ws_client::{ExchangeStream, WebSocket, WebSocketClient, WsError, WsMessage};

use crate::{
    data::{
        exchange::{Connector, StreamSelector},
        models::{event::MarketEvent, subs::Subscription, SubKind},
        transformer::Transformer,
    },
    error::SocketError,
};

pub const START_RECONNECTION_BACKOFF_MS: u64 = 125;

pub async fn connect<Exchange, StreamKind>(
    exchange_sub: Vec<Subscription<Exchange, StreamKind>>,
    exchange_tx: UnboundedSender<MarketEvent<StreamKind::Event>>,
    // exchange_tx: UnboundedSender<<Exchange::StreamTransformer as Transformer>::Output>,
) -> SocketError
where
    Exchange: Connector + Send + StreamSelector<Exchange, StreamKind>,
    StreamKind: SubKind,
    // StreamKind::Event: Into<<Exchange::StreamTransformer as Transformer>::Output>,
    <Exchange::StreamTransformer as Transformer>::Output: Into<MarketEvent<StreamKind::Event>>, // StreamKind::Event: Into<<Exchange::StreamTransformer as Transformer>::Output>
                                                                                                // ExchangeStream<Exchange, StreamKind, Exchange::StreamTransformer>: Stream,
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
                    println!("--- polled next ---");
                    println!("{:#?}", market_event);
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

/*----- */
// WebSocket message parser
/*----- */
pub trait StreamParser {
    type Stream: Stream;
    type Message;
    type Error;

    fn parse<Output>(
        input: Result<Self::Message, Self::Error>,
    ) -> Option<Result<Output, SocketError>>
    where
        Output: DeserializeOwned;
}
pub struct WebSocketParser;

impl StreamParser for WebSocketParser {
    type Stream = WebSocket;
    type Message = WsMessage;
    type Error = WsError;

    fn parse<Output>(
        input: Result<Self::Message, Self::Error>,
    ) -> Option<Result<Output, SocketError>>
    where
        Output: DeserializeOwned,
    {
        match input {
            Ok(ws_message) => match ws_message {
                WsMessage::Text(text) => process_text(text),
                WsMessage::Binary(binary) => process_binary(binary),
                WsMessage::Ping(ping) => process_ping(ping),
                WsMessage::Pong(pong) => process_pong(pong),
                WsMessage::Close(close_frame) => process_close_frame(close_frame),
                WsMessage::Frame(frame) => process_frame(frame),
            },
            Err(ws_err) => Some(Err(SocketError::WebSocketError(ws_err))),
        }
    }
}

pub fn parse<Output>(input: WsMessage) -> Option<Result<Output, SocketError>>
where
    Output: DeserializeOwned,
{
    match input {
        WsMessage::Text(text) => process_text(text),
        WsMessage::Binary(binary) => process_binary(binary),
        WsMessage::Ping(ping) => process_ping(ping),
        WsMessage::Pong(pong) => process_pong(pong),
        WsMessage::Close(close_frame) => process_close_frame(close_frame),
        WsMessage::Frame(frame) => process_frame(frame),
    }
}

pub fn process_text<ExchangeMessage>(
    payload: String,
) -> Option<Result<ExchangeMessage, SocketError>>
where
    ExchangeMessage: DeserializeOwned,
{
    Some(
        serde_json::from_str::<ExchangeMessage>(&payload)
            .map_err(|error| SocketError::Deserialise { error, payload }),
    )
}

pub fn process_binary<ExchangeMessage>(
    payload: Vec<u8>,
) -> Option<Result<ExchangeMessage, SocketError>>
where
    ExchangeMessage: DeserializeOwned,
{
    Some(
        serde_json::from_slice::<ExchangeMessage>(&payload).map_err(|error| {
            SocketError::Deserialise {
                error,
                payload: String::from_utf8(payload).unwrap_or_else(|x| x.to_string()),
            }
        }),
    )
}

pub fn process_ping<ExchangeMessage>(
    ping: Vec<u8>,
) -> Option<Result<ExchangeMessage, SocketError>> {
    format!("{:#?}", ping);
    None
}

pub fn process_pong<ExchangeMessage>(
    pong: Vec<u8>,
) -> Option<Result<ExchangeMessage, SocketError>> {
    format!("{:#?}", pong);
    None
}

pub fn process_close_frame<ExchangeMessage>(
    close_frame: Option<CloseFrame<'_>>,
) -> Option<Result<ExchangeMessage, SocketError>> {
    let close_frame = format!("{:?}", close_frame);
    Some(Err(SocketError::Terminated(close_frame)))
}

pub fn process_frame<ExchangeMessage>(
    frame: Frame,
) -> Option<Result<ExchangeMessage, SocketError>> {
    format!("{:?}", frame);
    None
}

pub fn is_websocket_disconnected(error: &WsError) -> bool {
    matches!(
        error,
        WsError::ConnectionClosed
            | WsError::AlreadyClosed
            | WsError::Io(_)
            | WsError::Protocol(ProtocolError::SendAfterClosing)
            | WsError::Protocol(ProtocolError::ResetWithoutClosingHandshake) // | WsError::Protocol(_)
    )
}
