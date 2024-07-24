use futures::Stream;
use serde::de::DeserializeOwned;
use tokio_tungstenite::tungstenite::{error::ProtocolError, protocol::{frame::Frame, CloseFrame}};

use crate::error::SocketError;

use super::{WebSocket, WsError, WsMessage};

/*----- */
// Websocket parser
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