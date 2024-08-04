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

use crate::data::{
    error::SocketError,
    event_models::SubKind,
    exchange::{Connector, Identifier, StreamSelector},
    shared::subscription_models::{ExchangeSubscription, Subscription},
    streams::validator::{SubscriptionValidator, WebSocketValidator},
    transformer::ExchangeTransformer,
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
    subs: &[Subscription<Exchange, StreamKind>],
) -> Result<ExchangeStream<Exchange::StreamTransformer>, SocketError>
where
    StreamKind: SubKind,
    Exchange: Connector + StreamSelector<Exchange, StreamKind> + Send + Clone + Debug + Sync,
    Exchange::StreamTransformer: ExchangeTransformer<Exchange::Stream, StreamKind>,
    Subscription<Exchange, StreamKind>:
        Identifier<Exchange::Channel> + Identifier<Exchange::Market> + Debug,
{
    // Convert subscription to internal subscription
    let exchange_subs = subs
        .iter()
        .map(|sub| ExchangeSubscription::new(sub))
        .collect::<Vec<_>>();

    // Make stream connection
    let mut tasks = Vec::new();
    let ws = connect_async(Exchange::url())
        .await
        .map(|(ws, _)| ws)
        .map_err(SocketError::WebSocketError);

    // Split WS and make into read and write
    let (mut ws_write, ws_read) = ws?.split();

    // Handle subscription
    if let Some(subcription) = Exchange::requests(&exchange_subs) {
        let _ = ws_write.send(subcription).await;
    }

    // Validate subscription
    let validated_stream =
        <WebSocketValidator as SubscriptionValidator>::validate(&exchange_subs, ws_read).await?;

    // Spawn custom ping handle (application level ping)
    if let Some(ping_interval) = Exchange::ping_interval() {
        let ping_handler = tokio::spawn(schedule_pings_to_exchange(ws_write, ping_interval));
        tasks.push(ping_handler);
    }

    // Make instruments to for transformer
    let instruments = subs
        .iter()
        .map(|sub| sub.instrument.clone())
        .collect::<Vec<_>>();

    let transformer = Exchange::StreamTransformer::new(&instruments).await?;

    Ok(ExchangeStream::new(validated_stream, transformer, tasks))
}

pub async fn schedule_pings_to_exchange(mut ws_write: WsWrite, ping_interval: PingInterval) {
    loop {
        sleep(Duration::from_secs(ping_interval.time)).await;
        let _ = ws_write
            .send(WsMessage::Text(ping_interval.message.to_string()))
            .await;
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
