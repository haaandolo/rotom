use std::io::Read;

use flate2::read::GzDecoder;
use futures::Stream;
use serde::de::DeserializeOwned;
use tokio_tungstenite::tungstenite::{
    error::ProtocolError,
    protocol::{frame::Frame, CloseFrame},
};

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
            Err(ws_err) => {
                if is_websocket_disconnected(&ws_err) {
                    Some(Err(SocketError::WebSocketDisconnected { error: ws_err }))
                } else {
                    Some(Err(SocketError::WebSocketError(ws_err)))
                }
            }
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

// Todo: make this more performant
pub fn process_binary<ExchangeMessage>(
    payload: Vec<u8>,
) -> Option<Result<ExchangeMessage, SocketError>>
where
    ExchangeMessage: DeserializeOwned,
{
    let mut decoder = GzDecoder::new(&payload[..]);
    let mut decoded = String::with_capacity(1000);

    if decoder.read_to_string(&mut decoded).is_ok() {
        // Sceario when compressed - use the decompressed string
        Some(
            serde_json::from_str::<ExchangeMessage>(decoded.as_str()).map_err(|error| {
                SocketError::Deserialise {
                    error,
                    payload: String::from_utf8(payload).unwrap_or_else(|x| x.to_string()),
                }
            }),
        )
    } else {
        // Scenario when is not compressed - use the original binary
        Some(
            serde_json::from_slice::<ExchangeMessage>(&payload).map_err(|error| {
                SocketError::Deserialise {
                    error,
                    payload: String::from_utf8(payload).unwrap_or_else(|x| x.to_string()),
                }
            }),
        )
    }
}

pub fn process_ping<ExchangeMessage>(
    _ping: Vec<u8>,
) -> Option<Result<ExchangeMessage, SocketError>> {
    None
}

pub fn process_pong<ExchangeMessage>(
    _pong: Vec<u8>,
) -> Option<Result<ExchangeMessage, SocketError>> {
    None
}

pub fn process_close_frame<ExchangeMessage>(
    close_frame: Option<CloseFrame<'_>>,
) -> Option<Result<ExchangeMessage, SocketError>> {
    let close_frame = format!("{:?}", close_frame);
    Some(Err(SocketError::Terminated(close_frame)))
}

pub fn process_frame<ExchangeMessage>(
    _frame: Frame,
) -> Option<Result<ExchangeMessage, SocketError>> {
    None
}

pub fn is_websocket_disconnected(error: &WsError) -> bool {
    matches!(
        error,
        WsError::ConnectionClosed
            | WsError::AlreadyClosed
            | WsError::Io(_)
            | WsError::Protocol(ProtocolError::SendAfterClosing)
            | WsError::Protocol(ProtocolError::ResetWithoutClosingHandshake) // WsError::Protocol(_)
    )
}
