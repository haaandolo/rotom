use crate::error::CustomErrors;
use futures::{
    future::join_all, stream::{SplitSink, SplitStream}, SinkExt, StreamExt
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

#[derive(Clone)]
pub struct WebSocketPayload {
    pub url: String,
    pub subscription: Option<Value>,
    pub ping_interval: Option<PingInterval>
}

#[derive(Clone)]
pub struct PingInterval {
    pub time: u64,
    pub message: Value
}

pub struct WebSocketBase;

impl WebSocketBase {
    pub async fn connect(payload: WebSocketPayload) {
        // try make socket connection
        let mut tasks = Vec::new();
        let ws = connect_async(payload.url)
            .await
            .map(|(ws, _)| ws)
            .map_err(CustomErrors::WebSocketConnectionError);

        // split socket into read and write and make channels 
        let (mut ws_sink, mut ws_stream) = ws.unwrap().split();
        let (ws_sink_tx, mut ws_sink_rx) = mpsc::unbounded_channel();


        // if payload exists send paylaod to write stream
        if let Some(subscription) = payload.subscription {
           ws_sink
            .send(Message::text(subscription.to_string()))
            .await
            .expect("Failed to send payload to WS");         
        }

        // if custom ping needs to be sent
        if let Some(ping_interval) = payload.ping_interval {
            let ping_interval = tokio::spawn(schedule_pings_to_exchange(ws_sink_tx, ping_interval));
            tasks.push(ping_interval)
        }

        let write_handle = tokio::spawn(async move {
            while let Some(msg) = ws_sink_rx.recv().await {
                ws_sink.send(msg).await.expect("Failed to send message");
            }
        });
        tasks.push(write_handle);

        let read_handle = tokio::spawn(async move {
            while let Some(msg) = ws_stream.next().await {
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
        });
        tasks.push(read_handle);

        let _ = join_all(tasks).await;

        // let ping_handle = tokio::spawn(schedule_pings_to_exchange(ws_sink_tx));

        // let _ = tokio::try_join!(vec![read_handle, write_handle);
    }
}

pub async fn schedule_pings_to_exchange(ws_sink_tx: mpsc::UnboundedSender<Message>, ping_interval: PingInterval) {
    loop {
        sleep(Duration::from_secs(ping_interval.time)).await;
        ws_sink_tx
            .send(Message::Text(ping_interval.message.to_string()))
            .unwrap();
    }
}
