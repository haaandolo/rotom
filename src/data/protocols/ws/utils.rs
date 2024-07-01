use futures::{SinkExt, Stream, StreamExt};
use serde::de::DeserializeOwned;
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{sleep, Duration},
};
use tokio_tungstenite::tungstenite::{
    error::ProtocolError,
    protocol::{frame::Frame, CloseFrame},
};

use crate::{
    data::{exchange_connector::Connector, Subscription},
    error::SocketError,
};

use super::{PingInterval, WebSocket, WebSocketClient, WsError, WsMessage, WsWrite};

pub const START_RECONNECTION_BACKOFF_MS: u64 = 125;

pub async fn try_connect<ExchangeConnector>(
    mut subscription: Subscription<ExchangeConnector>,
    exchange_tx: UnboundedSender<ExchangeConnector::Output>,
) where
    ExchangeConnector: Connector + std::marker::Send,
{
    let mut _connection_attempt: u32 = 0;
    let mut _backoff_ms: u64 = START_RECONNECTION_BACKOFF_MS;

    let mut ws_client = WebSocketClient::new(
        subscription.connector.url(),
        subscription.connector.requests(&subscription.instruments),
        subscription.connector.ping_interval(),
    );

    loop {
        _connection_attempt += 1;
        _backoff_ms *= 2;

        // Attempt to connect to the stream
        let mut stream = match ws_client.connect().await {
            Ok(stream) => {
                _connection_attempt = 0;
                _backoff_ms = START_RECONNECTION_BACKOFF_MS;
                stream
            }
            Err(error) => {
                if _connection_attempt == 1 {
                    panic!("First connection attemp failed with error: {:#?}", error);
                // CHANGE THIS PANIC
                } else {
                    continue;
                }
            }
        };

        // Validate subscriptions
        if let Some(Ok(WsMessage::Text(message))) = &stream.ws_read.next().await {
            let subscription_sucess = subscription
                .connector
                .validate_subscription(message.to_owned(), &subscription.instruments);

            println!("{:#?}", subscription_sucess);

            if !subscription_sucess {
                // break SocketError::Subscribe(format!("Subscription failed"))
                panic!("Subscription failed") // CHANGE THIS
            }
        };

        // Read from stream and send via channel, but if error occurs, attempt reconnection
        while let Some(message) = stream.ws_read.next().await {
            match message {
                Ok(ws_message) => {
                    let msg = match ws_message {
                        WsMessage::Text(text) => {
                            process_text::<<ExchangeConnector as Connector>::Input>(text)
                        }
                        WsMessage::Binary(binary) => process_binary(binary),
                        WsMessage::Ping(ping) => process_ping(ping),
                        WsMessage::Pong(pong) => process_pong(pong),
                        WsMessage::Close(close_frame) => process_close_frame(close_frame),
                        WsMessage::Frame(frame) => process_frame(frame),
                    }
                    .unwrap()
                    .unwrap();

                    let test = subscription.connector.transform(msg);
                    exchange_tx.send(test).expect("Failed to send message"); // CHANGE THIS
                }
                Err(error) => {
                    if is_websocket_disconnected(&error) {
                        // SHOULD CHANGE THIS TO SEE WHAT ERRORS ACTUAL MEANS RECONNECTING
                        stream.cancel_running_tasks()
                    }
                } // ADD Error for if sequence is broken then have to restart
            }
        }

        // Wait a certain ms before trying to reconnect
        sleep(Duration::from_millis(_backoff_ms)).await;
    }
}

/*----- */
// Stream Parser
/*----- */
pub struct WebSocketParser;

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

pub async fn schedule_pings_to_exchange(mut ws_write: WsWrite, ping_interval: PingInterval) {
    loop {
        sleep(Duration::from_secs(ping_interval.time)).await;
        ws_write
            .send(WsMessage::Text(ping_interval.message.to_string()))
            .await
            .expect("Failed to send ping to ws");
    }
}
