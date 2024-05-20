use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::{net::TcpStream, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
pub struct WebSocketClient {
    pub write_stream: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub read_stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

// impl Default for WebSocketClient {
//     async fn default() -> Self {
//         let (ws_stream, _) = connect_async("www...").await.expect("Failed to connect to ws");
//         let (write, read) = ws_stream.split();
//         WebSocketClient {
//             write_stream: write,
//             read_stream: read,
//         }
//     }
// }

impl WebSocketClient {
    pub async fn new(url: String, payload: Value) -> Self {
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to ws");
        let (mut write, read) = ws_stream.split();
        write.send(Message::Text(payload.to_string())).await.unwrap();
        Self {
            write_stream: write,
            read_stream: read,
        }
    }

    pub async fn set_ping(&mut self, interval: u64) {
        let ping = json!({ "event": "ping" });
        loop {
            sleep(Duration::from_secs(interval)).await;
            self.write_stream
                .send(Message::Text(ping.to_string()))
                .await
                .expect("Failed to set ping");
        }
    }

    pub async fn write_stream(&mut self, msg: Value) {
        self.write_stream
            .send(Message::Text(msg.to_string()))
            .await
            .expect("Failed to send message")
    }

    pub async fn read_stream(&mut self) {
        while let Some(msg) = self.read_stream.next().await {
            println!("{:#?}", msg);
            match msg {
                Ok(Message::Text(stream)) => {
                    let stream: Value =
                        serde_json::from_str(&stream).expect("Failed to convert Message to Json");
                    println!("{:#?}", stream) // stream["channel"]
                }
                Ok(Message::Ping(_)) => {}
                _ => (),
            }
        }
    }
}
