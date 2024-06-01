use crate::error::SocketError;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::{
    net::TcpStream,
    sync::mpsc,
    time::{sleep, Duration},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        error::ProtocolError,
        protocol::{frame::Frame, CloseFrame},
    },
    MaybeTlsStream, WebSocketStream,
};

pub type WsMessage = tokio_tungstenite::tungstenite::Message;
pub type WsError = tokio_tungstenite::tungstenite::Error;
pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>;
pub type FuturesTokio = tokio::task::JoinHandle<()>;


// Websocket base
#[derive(Clone)]
pub struct WebSocketPayload<'a> {
    pub url: &'a str,
    pub subscription: Option<Value>,
    pub ping_interval: Option<PingInterval>,
}

#[derive(Clone)]
pub struct PingInterval {
    pub time: u64,
    pub message: Value,
}

pub struct WebSocketBase;

impl WebSocketBase {
    pub async fn connect(
        payload: WebSocketPayload<'_>,
    ) -> Result<(WsRead, Vec<FuturesTokio>), SocketError> {
        // Vec of futures to run
        let mut tasks = Vec::new();

        // Make connection
        let ws = connect_async(payload.url)
            .await
            .map(|(ws, _)| ws)
            .map_err(SocketError::WebSocketError);

        // Split WS and make channels
        let (mut ws_write, ws_stream) = ws?.split();
        let (ws_sink_tx, mut ws_sink_rx) = mpsc::unbounded_channel();

        // Handle subscription
        if let Some(subscription) = payload.subscription {
            ws_write
                .send(WsMessage::text(subscription.to_string()))
                .await?
        }

        // Handle custom ping
        if let Some(ping_interval) = payload.ping_interval {
            let ping_handler = tokio::spawn(schedule_pings_to_exchange(ws_sink_tx, ping_interval));
            tasks.push(ping_handler);
        }

        // Hand writes using channel
        let write_handler = tokio::spawn(async move {
            while let Some(msg) = ws_sink_rx.recv().await {
                let _ = ws_write
                    .send(msg)
                    .await
                    .map_err(SocketError::WebSocketError);
            }
        });
        tasks.push(write_handler);

        Ok((ws_stream, tasks))
    }
}

pub async fn schedule_pings_to_exchange(
    ws_sink_tx: mpsc::UnboundedSender<WsMessage>,
    ping_interval: PingInterval,
) {
    loop {
        sleep(Duration::from_secs(ping_interval.time)).await;
        ws_sink_tx
            .send(WsMessage::Text(ping_interval.message.to_string()))
            .unwrap();
    }
}

// Stream parser
pub trait StreamParser {
    type Message;
    type Error;

    fn parse<Output>(
        input: Result<Self::Message, Self::Error>,
    ) -> Option<Result<Output, SocketError>>
    where
        Output: DeserializeOwned;
}

impl StreamParser for WebSocketBase {
    type Message = WsMessage;
    type Error = tokio_tungstenite::tungstenite::Error;

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
    format!("{:?}", pong);
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
    )
}
