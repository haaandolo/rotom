use crate::error::CustomErrors;
use futures::{
    channel::mpsc::UnboundedReceiver,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::{json, Value};
use tokio::{
    net::TcpStream,
    sync::mpsc,
    time::{sleep, Duration},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub struct WebSocketBase {
    pub ws: WebSocket,
    pub is_connected: bool,
}

impl WebSocketBase {
    pub async fn connect(url: &str, payload: Value) {
        let ws = connect_async(url)
            .await
            .map(|(ws, _)| ws)
            .map_err(CustomErrors::WebSocketConnectionError);

        let (mut ws_sink, ws_stream) = ws.unwrap().split();
        // // let (ws_sink_tx, ws_sink_rx) = mpsc::unbounded_channel();

        ws_sink
            .send(Message::text(payload.to_string()))
            .await
            .unwrap();

        let ping_handle = tokio::spawn(schedule_pings_to_exchange(ws_sink));
        let read_handle = tokio::spawn(read_stream(ws_stream));
        // let write_handle = tokio::spawn(write_stream(ws_sink_rx, ws_sink));

        let _ = tokio::try_join!(ping_handle, read_handle);
    }
}

pub async fn schedule_pings_to_exchange(mut ws_sink: WsWrite) {
    loop {
        sleep(Duration::from_secs(20)).await;
        let ping = json!({"event": "ping"});
        ws_sink
            .send(Message::text(ping.to_string()))
            .await
            .expect("Failed to send message");
    }
}

pub async fn read_stream(mut read: WsRead) {
    while let Some(msg) = &read.next().await {
        match msg {
            Ok(Message::Text(stream)) => {
                // let stream: Value =
                //     serde_json::from_str(stream).expect("Failed to convert Message to Json");
                println!("{:#?}", stream) // stream["channel"]
            }
            Ok(Message::Ping(_)) => {}
            _ => (),
        }
    }
}

// pub async fn write_stream(ws_sink_rx: mpsc::UnboundedReceiver<Message>, mut write: WsWrite) {
//     while let Some(msg) = ws_sink_rx.next().await {
//         write.send(msg)
//     }
// }
