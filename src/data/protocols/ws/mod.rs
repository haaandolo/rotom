pub mod ws_client;

use futures::StreamExt;
use serde::{de::DeserializeOwned, Deserialize};
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{sleep, Duration},
};
use tokio_tungstenite::tungstenite::{
    error::ProtocolError,
    protocol::{frame::Frame, CloseFrame},
};
use ws_client::{WebSocketClient, WsError, WsMessage};

use crate::{
    data::{exchange_connector::Connector, Subscription},
    error::SocketError,
};

pub const START_RECONNECTION_BACKOFF_MS: u64 = 125;

pub async fn connect<StreamKind, ExchangeConnector>(
    mut subscription: Subscription<ExchangeConnector>,
    exchange_tx: UnboundedSender<StreamKind>,
) -> SocketError
where
    ExchangeConnector: Connector + Send,
    StreamKind: for<'de> Deserialize<'de>,
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
        let mut stream = match ws_client.create_websocket().await {
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
            let subscription_sucess = subscription
                .connector
                .validate_subscription(message.to_owned(), &subscription.instruments);

            if !subscription_sucess {
                break SocketError::Subscribe(String::from("Subscription failed"));
            }
        };

        // Read from stream and send via channel, but if error occurs, attempt reconnection
        while let Some(message) = stream.ws_read.next().await {
            match message {
                Ok(ws_message) => {
                    let deserialized_message = match parse::<StreamKind>(ws_message) {
                        Some(Ok(exchange_message)) => exchange_message,
                        Some(Err(error)) => {
                            println!("Failed to deserialise WsMessage: {:#?}", &error); // Log this error
                                                                                        // Defs dont return this as crashed the whole program
                            return SocketError::Subscribe(format!(
                                "Failed to deserialise WsMessage: {}",
                                error
                            ));
                        }
                        None => continue,
                    };

                    exchange_tx
                        .send(deserialized_message)
                        .expect("failed to send message");
                    // let event = subscription.connector.transform(deserialized_message);
                    // exchange_tx.send(event).expect("failed to send message");
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
// WebSocket message parser
/*----- */
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
