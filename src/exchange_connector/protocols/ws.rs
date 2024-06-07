use crate::{error::SocketError, exchange_connector::Exchange};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::Value;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedReceiver},
    time::{sleep, Duration},
};
use tokio_tungstenite::{
    connect_async, tungstenite::error::ProtocolError, MaybeTlsStream, WebSocketStream,
};

pub type WsMessage = tokio_tungstenite::tungstenite::Message;
pub type WsError = tokio_tungstenite::tungstenite::Error;
pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>;
pub type JoinHandle = tokio::task::JoinHandle<()>;

/*---------- */
// Models
/*---------- */
#[derive(Clone, Debug)]
pub struct PingInterval {
    pub time: u64,
    pub message: Value,
}

pub struct ExchangeStream {
    pub stream: UnboundedReceiver<WsMessage>,
    pub tasks: Vec<JoinHandle>,
}

impl ExchangeStream {
    pub fn cancel_running_tasks(&self) {
        self.tasks.iter().for_each(|task| {
            task.abort();
        })
    }
}

#[derive(Debug, Clone)]
pub struct WebSocketPayload {
    pub exchange: Exchange,
    pub url: String,
    pub subscription: Option<WsMessage>,
    pub ping_interval: Option<PingInterval>,
}

/*---------- */
// WebSocket
/*---------- */
#[derive(Debug)]
pub struct WebSocketClient;

impl WebSocketClient {
    pub async fn connect(self, payload: WebSocketPayload) -> Result<ExchangeStream, SocketError> {
        // Make connection
        let mut _tasks = Vec::new();
        let ws = connect_async(payload.url)
            .await
            .map(|(ws, _)| ws)
            .map_err(SocketError::WebSocketError);

        // Split WS and make channels
        let (mut ws_write, ws_stream) = ws?.split();
        let (ws_sink_tx, ws_sink_rx) = mpsc::unbounded_channel();

        // Handle subscription
        if let Some(subcription) = payload.subscription {
            ws_write.send(subcription).await?
        }

        // Handle writes
        let write_handler = tokio::spawn(write_to_ws(ws_sink_rx, ws_write));
        _tasks.push(write_handler);

        // Handle custom ping
        if let Some(ping_interval) = payload.ping_interval {
            let ping_handler = tokio::spawn(schedule_pings_to_exchange(
                ws_sink_tx.clone(),
                ping_interval,
            ));
            _tasks.push(ping_handler);
        }

        // Handle read by spawning channel
        let (ws_stream_tx, ws_stream_rx) = mpsc::unbounded_channel();
        let read_handler = tokio::spawn(read_from_ws(ws_stream_tx, ws_stream));
        _tasks.push(read_handler);

        let exchange_stream = ExchangeStream {
            stream: ws_stream_rx,
            tasks: _tasks,
        };

        Ok(exchange_stream)
    }
}

async fn read_from_ws(ws_stream_tx: mpsc::UnboundedSender<WsMessage>, mut ws_stream: WsRead) {
    while let Some(msg) = ws_stream.next().await {
        ws_stream_tx
            .clone()
            .send(msg.unwrap())
            .expect("Failed to send msg from WS")
    }
}

async fn write_to_ws(mut ws_sink_rx: mpsc::UnboundedReceiver<WsMessage>, mut ws_sink: WsWrite) {
    while let Some(msg) = ws_sink_rx.recv().await {
        let _ = ws_sink.send(msg).await.map_err(SocketError::WebSocketError);
    }
}

async fn schedule_pings_to_exchange(
    ws_sink_tx: mpsc::UnboundedSender<WsMessage>,
    ping_interval: PingInterval,
) {
    loop {
        sleep(Duration::from_secs(ping_interval.time)).await;
        ws_sink_tx
            .send(WsMessage::Text(ping_interval.message.to_string()))
            .expect("Failed to send ping to ws");
    }
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

/*---------- */
// STREAM PARSER
/*---------- */
// STREAM PARSER
// use tokio_tungstenite::tungstenite::{error::ProtocolError, protocol::{frame::Frame, CloseFrame}};
// use serde::de::DeserializeOwned;

// pub trait StreamParser {
//     type Message;
//     type Error;

//     fn parse<Output>(
//         input: Result<Self::Message, Self::Error>,
//     ) -> Option<Result<Output, SocketError>>
//     where
//         Output: DeserializeOwned;
// }

// impl StreamParser for WebSocketBase {
//     type Message = WsMessage;
//     type Error = tokio_tungstenite::tungstenite::Error;

//     fn parse<Output>(
//         input: Result<Self::Message, Self::Error>,
//     ) -> Option<Result<Output, SocketError>>
//     where
//         Output: DeserializeOwned,
//     {
//         match input {
//             Ok(ws_message) => match ws_message {
//                 WsMessage::Text(text) => process_text(text),
//                 WsMessage::Binary(binary) => process_binary(binary),
//                 WsMessage::Ping(ping) => process_ping(ping),
//                 WsMessage::Pong(pong) => process_pong(pong),
//                 WsMessage::Close(close_frame) => process_close_frame(close_frame),
//                 WsMessage::Frame(frame) => process_frame(frame),
//             },
//             Err(ws_err) => Some(Err(SocketError::WebSocketError(ws_err))),
//         }
//     }
// }

// // pub fn parse<Output>(input: Result<WsMessage, WsError>) -> Option<Result<Output, SocketError>>
// // where
// //     Output: DeserializeOwned,
// // {
// //     match input {
// //         Ok(ws_message) => match ws_message {
// //             WsMessage::Text(text) => process_text(text),
// //             WsMessage::Binary(binary) => process_binary(binary),
// //             WsMessage::Ping(ping) => process_ping(ping),
// //             WsMessage::Pong(pong) => process_pong(pong),
// //             WsMessage::Close(close_frame) => process_close_frame(close_frame),
// //             WsMessage::Frame(frame) => process_frame(frame),
// //         },
// //         Err(ws_err) => Some(Err(SocketError::WebSocketError(ws_err))),
// //     }
// // }

// pub fn process_text<ExchangeMessage>(
//     payload: String,
// ) -> Option<Result<ExchangeMessage, SocketError>>
// where
//     ExchangeMessage: DeserializeOwned,
// {
//     Some(
//         serde_json::from_str::<ExchangeMessage>(&payload)
//             .map_err(|error| SocketError::Deserialise { error, payload }),
//     )
// }

// pub fn process_binary<ExchangeMessage>(
//     payload: Vec<u8>,
// ) -> Option<Result<ExchangeMessage, SocketError>>
// where
//     ExchangeMessage: DeserializeOwned,
// {
//     Some(
//         serde_json::from_slice::<ExchangeMessage>(&payload).map_err(|error| {
//             SocketError::Deserialise {
//                 error,
//                 payload: String::from_utf8(payload).unwrap_or_else(|x| x.to_string()),
//             }
//         }),
//     )
// }

// pub fn process_ping<ExchangeMessage>(
//     ping: Vec<u8>,
// ) -> Option<Result<ExchangeMessage, SocketError>> {
//     format!("{:#?}", ping);
//     None
// }

// pub fn process_pong<ExchangeMessage>(
//     pong: Vec<u8>,
// ) -> Option<Result<ExchangeMessage, SocketError>> {
//     format!("{:?}", pong);
//     None
// }

// pub fn process_close_frame<ExchangeMessage>(
//     close_frame: Option<CloseFrame<'_>>,
// ) -> Option<Result<ExchangeMessage, SocketError>> {
//     let close_frame = format!("{:?}", close_frame);
//     Some(Err(SocketError::Terminated(close_frame)))
// }

// pub fn process_frame<ExchangeMessage>(
//     frame: Frame,
// ) -> Option<Result<ExchangeMessage, SocketError>> {
//     format!("{:?}", frame);
//     None
// }

// pub fn is_websocket_disconnected(error: &WsError) -> bool {
//     matches!(
//         error,
//         WsError::ConnectionClosed
//             | WsError::AlreadyClosed
//             | WsError::Io(_)
//             | WsError::Protocol(ProtocolError::SendAfterClosing)
//     )
// }
