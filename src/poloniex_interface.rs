use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, WebSocketStream};

pub struct PoloniexInterface {}

impl Default for PoloniexInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl PoloniexInterface {

    pub fn new() -> Self {
        Self {}
    }

    pub async fn stream_data(&self, tickers: Vec<&str>) {
        let url = "wss://ws.poloniex.com/ws/public";
        let payload = json!({
            "event": "subscribe",
            "channel": ["book_lv2"], //book_lv2
            "symbols": tickers
        });

        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to ws");
        let (mut write, mut read) = ws_stream.split();

        write
            .send(Message::Text(payload.to_string()))
            .await
            .expect("Failed to send message");

        let ws_read = tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(stream)) => {
                        let stream: Value = serde_json::from_str(&stream)
                            .expect("Failed to convert Message to Json");
                        println!("{:#?}", stream) // stream["channel"] 
                    }
                    Ok(Message::Ping(_)) => {}
                    _ => (),
                }
            }
        });

        tokio::select! {
            _ = ws_read => (),
            _ = self.send_ping(&mut write) => (),
        };
    }

    async fn send_ping(
        &self,
        write: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
    ) {
        let ping = json!({
            "event": "ping"
        });
        loop {
            sleep(Duration::from_secs(20)).await;
            write
                .send(Message::Text(ping.to_string()))
                .await
                .expect("Failed to send message");
        }
    }
}
