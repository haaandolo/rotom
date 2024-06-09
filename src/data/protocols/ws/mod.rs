pub mod utils;

use crate::error::SocketError;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, MaybeTlsStream, WebSocketStream,
};
use utils::schedule_pings_to_exchange;

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
    pub ws_read: WsRead,
    pub tasks: Vec<JoinHandle>,
}

impl ExchangeStream {
    pub fn cancel_running_tasks(&self) {
        self.tasks.iter().for_each(|task| {
            task.abort();
        })
    }
}

/*---------- */
// WebSocket
/*---------- */
#[derive(Debug)]
pub struct WebSocketClient {
    pub url: String,
    pub subscription: Option<WsMessage>,
    pub ping_interval: Option<PingInterval>,
}

impl WebSocketClient {
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
    pub async fn connect(&mut self) -> Result<ExchangeStream, SocketError> {
        // Make connection
        let mut _tasks = Vec::new();
        let ws = connect_async(self.url.clone())
            .await
            .map(|(ws, _)| ws)
            .map_err(SocketError::WebSocketError);

        // Split WS and make channels
        let (mut ws_write, ws_stream) = ws?.split();

        // Handle subscription
        if let Some(subcription) = self.subscription.clone() {
            ws_write
                .send(subcription)
                .await
                .expect("Failed to send subscription")
        }

        // Spawn custom ping handle (application level ping)
        if let Some(ping_interval) = self.ping_interval.clone() {
            let ping_handler = tokio::spawn(schedule_pings_to_exchange(ws_write, ping_interval));
            _tasks.push(ping_handler);
        }

        Ok(ExchangeStream {
            ws_read: ws_stream,
            tasks: _tasks,
        })
    }
}
