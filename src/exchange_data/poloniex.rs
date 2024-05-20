pub mod poloniex_data {

    use futures_util::stream::{SplitSink, SplitStream};
    use futures_util::{SinkExt, StreamExt};
    use serde_json::{json, Value};
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio::time::{sleep, Duration};
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::{connect_async, WebSocketStream};
    // use tokio::sync::mpsc;

    async fn send_ping(
        write: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
    ) {
        let ping = json!({ "event": "ping" });
        loop {
            sleep(Duration::from_secs(20)).await;
            write
                .send(Message::Text(ping.to_string()))
                .await
                .expect("Failed to send message");
        }
    }

    async fn event_read_handler(
        mut read: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>,
    ) {
        while let Some(msg) = &read.next().await {
            match msg {
                Ok(Message::Text(stream)) => {
                    let stream: Value =
                        serde_json::from_str(stream).expect("Failed to convert Message to Json");
                    println!("{:#?}", stream) // stream["channel"]
                }
                Ok(Message::Ping(_)) => {}
                _ => (),
            }
        }
    }

    // async fn write_to_socket(tx: mpsc::UnboundedSender<Message>, msg: Value) {
    //     tx.send(Message::Text(msg.to_string()));
    // }

    pub async fn stream_data(tickers: Vec<&str>, channels: Vec<&str>) {
        let url = "wss://ws.poloniex.com/ws/public";
        let payload = json!({
            "event": "subscribe",
            "channel": channels,
            "symbols": tickers
        });
        // let (tx, rx) = mpsc::unbounded_channel();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to ws");
        let (mut write, read) = ws_stream.split();

        write
            .send(Message::Text(payload.to_string()))
            .await
            .expect("Failed to send message");

        tokio::select! {
            _ = event_read_handler(read) => (),
            _ = send_ping(&mut write) => (),
        };
    }
}
