pub mod poloniex_data {

    use futures_util::stream::{SplitSink, SplitStream};
    use futures_util::{SinkExt, StreamExt};
    use serde_json::{json, Value};
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio::time::{sleep, Duration};
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::{connect_async, WebSocketStream};

    async fn send_ping(
        write: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
    ) {
        let ping = json!({ "event": "ping" });
        loop {
            sleep(Duration::from_secs(25)).await;
            write
                .send(Message::Text(ping.to_string()))
                .await
                .expect("Failed to send message");
        }
    }

    async fn read_stream(
        mut read: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>,
    ) {
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

    async fn write_socket(
        write: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
        msg: Value,
    ) {
        write
            .send(Message::Text(msg.to_string()))
            .await
            .expect("Failed to send message")
    }

    pub async fn stream_data(tickers: Vec<&str>, channels: Vec<&str>) {
        let url = "wss://ws.poloniex.com/ws/public";
        let payload = json!({
            "event": "subscribe",
            "channel": channels,
            "symbols": tickers
        });
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to ws");
        let (mut write, read) = ws_stream.split();

        write_socket(&mut write, payload).await;

        let ping = tokio::spawn(async move {
            send_ping(&mut write).await;
        });

        let read = tokio::spawn(async move {
            read_stream(read).await;
        });

        let _ = tokio::try_join!(ping, read);

    }
}
