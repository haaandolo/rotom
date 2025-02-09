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
use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, info};

use crate::{
    error::SocketError,
    exchange::{Identifier, PublicStreamConnector, StreamSelector},
    model::SubKind,
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
pub struct WebSocketClient;

impl WebSocketClient {
    pub async fn init<Exchange, StreamKind>(
        subs: &[Subscription<Exchange, StreamKind>],
    ) -> Result<ExchangeStream<Exchange::StreamTransformer>, SocketError>
    where
        StreamKind: SubKind,
        Exchange: PublicStreamConnector
            + StreamSelector<Exchange, StreamKind>
            + Send
            + Clone
            + Debug
            + Sync,
        Exchange::StreamTransformer: ExchangeTransformer<Exchange, Exchange::Stream, StreamKind>,
        Subscription<Exchange, StreamKind>:
            Identifier<Exchange::Channel> + Identifier<Exchange::Market> + Debug,
    {
        // Convert subscription to internal subscription
        let exchange_id = Exchange::ID;
        let exchange_subs = subs
            .iter()
            .map(|sub| ExchangeSubscription::new(sub))
            .collect::<Vec<_>>();

        // Make stream connection
        let mut tasks = Vec::new();
        let ws = connect(Exchange::url().into()).await?;

        // Split WS and make into read and write
        let (mut ws_write, ws_read) = ws.split();

        // Handle subscription
        if let Some(subcription) = Exchange::requests(&exchange_subs) {
            let _ = ws_write.send(subcription).await;
        }

        // Validate subscription
        let validated_stream = WebSocketValidator::validate(&exchange_subs, ws_read).await?;

        // Spawn custom ping handle (application level ping)
        if let Some(ping_interval) = Exchange::ping_interval() {
            let ping_handler = tokio::spawn(schedule_pings_to_exchange(ws_write, ping_interval));
            tasks.push(ping_handler);
        }

        // Make instruments for transformer
        let instruments = subs
            .iter()
            .map(|sub| sub.instrument.clone())
            .collect::<Vec<_>>();

        let transformer = Exchange::StreamTransformer::new(&exchange_subs).await?;

        // Log connection success message
        info!(
            exchange = %exchange_id,
            message = "Subscribed to WebSocket",
            instruments = ?instruments
        );

        Ok(ExchangeStream::new(validated_stream, transformer, tasks))
    }
}

pub async fn schedule_pings_to_exchange(mut ws_write: WsWrite, ping_interval: PingInterval) {
    loop {
        sleep(Duration::from_secs(ping_interval.time)).await;
        let _ = ws_write
            .send(WsMessage::Text(ping_interval.message.to_string()))
            .await;
    }
}

pub async fn connect<R>(request: R) -> Result<WebSocket, SocketError>
where
    R: IntoClientRequest + Unpin + Debug,
{
    debug!(?request, "attempting to establish WebSocket connection");
    connect_async(request)
        .await
        .map(|(websocket, _)| websocket)
        .map_err(SocketError::WebSocketError)
}

/*----- */
// Models
/*----- */
#[derive(Clone, Debug)]
pub struct PingInterval {
    pub time: u64,
    pub message: Value,
}
