pub mod poll_next;
pub mod ws_parser;

use std::fmt::Debug;

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use poll_next::ExchangeStream;
use serde_json::Value;
use tokio::{net::TcpStream, time::sleep, time::Duration};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::{
    data::{
        exchange::{Connector, StreamSelector},
        model::{subs::Instrument, SubKind},
        transformer::ExchangeTransformer,
    },
    error::SocketError,
};

/*----- */
// Convenient types
/*----- */
pub type WsMessage = tokio_tungstenite::tungstenite::Message;
pub type WsError = tokio_tungstenite::tungstenite::Error;
pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>;
pub type JoinHandle = tokio::task::JoinHandle<()>;

/*----- */
// Create websocket
/*----- */
pub async fn create_websocket<Exchange, StreamKind>(
    subs: &[Instrument],
) -> Result<ExchangeStream<Exchange::StreamTransformer>, SocketError>
where
    StreamKind: SubKind,
    Exchange: Connector + StreamSelector<Exchange, StreamKind> + Send,
    Exchange::StreamTransformer: ExchangeTransformer<Exchange::Stream, StreamKind> + Debug, // DEL
{
    // Make connection
    let mut tasks = Vec::new();
    let ws = connect_async(Exchange::url())
        .await
        .map(|(ws, _)| ws)
        .map_err(SocketError::WebSocketError);

    // Split WS and make channels
    let (mut ws_write, ws_read) = ws?.split();

    // Handle subscription
    if let Some(subcription) = Exchange::requests(subs) {
        ws_write
            .send(subcription)
            .await
            .expect("Failed to send subscription")
    }

    // Spawn custom ping handle (application level ping)
    if let Some(ping_interval) = Exchange::ping_interval() {
        let ping_handler = tokio::spawn(schedule_pings_to_exchange(ws_write, ping_interval));
        tasks.push(ping_handler);
    }

    let transformer = Exchange::StreamTransformer::new(subs).await?;

    Ok(ExchangeStream::new(ws_read, transformer, tasks))
}

pub async fn schedule_pings_to_exchange(mut ws_write: WsWrite, ping_interval: PingInterval) {
    loop {
        sleep(Duration::from_secs(ping_interval.time)).await;
        ws_write
            .send(WsMessage::Text(ping_interval.message.to_string()))
            .await
            .expect("Failed to send ping to ws");
    }
}

/*----- */
// Models
/*----- */
#[derive(Clone, Debug)]
pub struct PingInterval {
    pub time: u64,
    pub message: Value,
}
