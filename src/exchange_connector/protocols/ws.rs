use crate::error::SocketError;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::Value;
use tokio::{
    net::TcpStream,
    sync::mpsc,
    time::{sleep, Duration},
};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub type WsMessage = tokio_tungstenite::tungstenite::Message;
pub type WsError = tokio_tungstenite::tungstenite::Error;
pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>;
pub type FuturesTokio = tokio::task::JoinHandle<()>;

/*---------- */
// Models
/*---------- */
#[derive(Clone, Debug)]
pub struct PingInterval {
    pub time: u64,
    pub message: Value,
}

pub struct ExchangeStream {
    pub stream: WsRead,
    pub tasks: Vec<FuturesTokio>,
}

/*---------- */
// WebSocket
/*---------- */
#[derive(Debug)]
pub struct WebSocketBase {
    pub url: String,
    pub subscription: Option<WsMessage>,
    pub ping_interval: Option<PingInterval>,
}

impl WebSocketBase {
    pub fn new(
        _url: String,
        _subscription: Option<WsMessage>,
        _ping_interval: Option<PingInterval>,
    ) -> Self {
        Self {
            url: _url,
            subscription: _subscription,
            ping_interval: _ping_interval,
        }
    }

    pub async fn connect(self) -> Result<ExchangeStream, SocketError> {
        // Make connection
        let mut _tasks = Vec::new();
        let ws = connect_async(self.url)
            .await
            .map(|(ws, _)| ws)
            .map_err(SocketError::WebSocketError);

        // Split WS and make channels
        let (mut ws_write, ws_stream) = ws?.split();
        let (ws_sink_tx, mut ws_sink_rx) = mpsc::unbounded_channel();

        // Handle writes
        let write_handler = tokio::spawn(async move {
            while let Some(msg) = ws_sink_rx.recv().await {
                let _ = ws_write
                    .send(msg)
                    .await
                    .map_err(SocketError::WebSocketError);
            }
        });
        _tasks.push(write_handler);

        // Handle subscription
        if let Some(subscription) = self.subscription {
            ws_sink_tx
                .clone()
                .send(subscription)
                .expect("Failed to send subscription to ws")
        }

        // Handle custom ping
        if let Some(ping_interval) = self.ping_interval {
            let ping_handler = tokio::spawn(schedule_pings_to_exchange(
                ws_sink_tx.clone(),
                ping_interval,
            ));
            _tasks.push(ping_handler);
        }

        let exchange_stream = ExchangeStream {
            stream: ws_stream,
            tasks: _tasks,
        };

        Ok(exchange_stream)
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
            .expect("Failed to send ping to ws");
    }
}

/*---------- */
// WebSocket Builder
/*---------- */
pub struct WebSocketBuilder {
    pub url: String,
    pub subscription: Option<WsMessage>,
    pub ping_interval: Option<PingInterval>,
}

impl WebSocketBuilder {
    pub fn new(_url: String) -> Self {
        Self {
            url: _url,
            subscription: None,
            ping_interval: None,
        }
    }

    pub fn set_url(&mut self, url: String) -> &mut Self {
        self.url = url;
        self
    }

    pub fn set_subscription(&mut self, subcription: WsMessage) -> &mut Self {
        self.subscription = Some(subcription);
        self
    }

    pub fn set_ping_interval(&mut self, ping_interval: Option<PingInterval>) -> &mut Self {
        self.ping_interval = ping_interval;
        self
    }

    pub async fn build(&mut self) -> Result<ExchangeStream, SocketError> {
        WebSocketBase::new(
            self.url.clone(),
            self.subscription.clone(),
            self.ping_interval.clone(),
        )
        .connect()
        .await
    }
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
