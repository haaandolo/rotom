use crate::error::{NetworkErrors, Result};
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
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

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
    pub async fn connect(payload: WebSocketPayload<'_>) -> Result<WsRead> {
        // Make connection
        let ws = connect_async(payload.url)
            .await
            .map(|(ws, _)| ws)
            .map_err(NetworkErrors::WebSocketConnectionError);

        // Split WS and make channels
        let (mut ws_write, ws_stream) = ws?.split();
        let (ws_sink_tx, mut ws_sink_rx) = mpsc::unbounded_channel();

        // Handle subscription
        if let Some(subscription) = payload.subscription {
            ws_write
                .send(Message::text(subscription.to_string()))
                .await?
        }

        // Handle custom ping
        if let Some(ping_interval) = payload.ping_interval {
            tokio::spawn(schedule_pings_to_exchange(ws_sink_tx, ping_interval));
        }

        // Hand writes using channel
        tokio::spawn(async move {
            while let Some(msg) = ws_sink_rx.recv().await {
                let _ = ws_write
                    .send(msg)
                    .await
                    .map_err(NetworkErrors::WebSocketConnectionError);
            }
        });

        Ok(ws_stream)
    }
}

pub async fn schedule_pings_to_exchange(
    ws_sink_tx: mpsc::UnboundedSender<Message>,
    ping_interval: PingInterval,
) {
    loop {
        sleep(Duration::from_secs(ping_interval.time)).await;
        ws_sink_tx
            .send(Message::Text(ping_interval.message.to_string()))
            .unwrap();
    }
}
