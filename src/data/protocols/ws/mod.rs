pub mod utils;

use std::{collections::VecDeque, fmt::Debug, pin::pin, str::FromStr, task::Poll};

use chrono::{DateTime, Utc};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, Stream, StreamExt,
};
use pin_project::pin_project;
use serde::{
    de::{self, DeserializeOwned},
    Deserialize,
};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::{frame::Frame, CloseFrame},
    MaybeTlsStream, WebSocketStream,
};
use utils::schedule_pings_to_exchange;

use crate::{
    data::{
        exchange_connector::binance::book::{BinanceMessage, BinanceSnapshot, BinanceTradeUpdate},
        shared::{de::de_u64_epoch_ms_as_datetime_utc, orderbook::event::Event},
    },
    error::SocketError,
};

pub type WsMessage = tokio_tungstenite::tungstenite::Message;
pub type WsError = tokio_tungstenite::tungstenite::Error;
pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type WsWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>;
pub type JoinHandle = tokio::task::JoinHandle<()>;

/*----- */
// Models
/*----- */
#[derive(Clone, Debug)]
pub struct PingInterval {
    pub time: u64,
    pub message: Value,
}

pub struct ExchangeStream {
    pub ws_read: ExchangeStream2<WsRead, StatefulTransformer>,
    pub tasks: Vec<JoinHandle>,
}

impl ExchangeStream {
    pub fn cancel_running_tasks(&self) {
        self.tasks.iter().for_each(|task| {
            task.abort();
        })
    }
}

/*----- */
// WebSocket
/*----- */
#[derive(Debug)]
pub struct WebSocketClient {
    pub url: String,
    pub subscription: Option<WsMessage>,
    pub ping_interval: Option<PingInterval>,
}

impl WebSocketClient {
    pub fn new(
        _url: String,
        _subscription: Option<WsMessage>,
        _ping_interval: Option<PingInterval>,
    ) -> Self {
        Self {
            url: _url,
            subscription: _subscription,
            ping_interval: _ping_interval,
        }
    }
    pub async fn connect(&mut self) -> Result<ExchangeStream, SocketError> {
        // Make connection
        let mut _tasks = Vec::new();
        let ws = connect_async(self.url.clone())
            .await
            .map(|(ws, _)| ws)
            .map_err(SocketError::WebSocketError);

        // Split WS and make channels
        let (mut ws_write, ws_stream) = ws?.split();

        // Handle subscription
        if let Some(subcription) = self.subscription.clone() {
            ws_write
                .send(subcription)
                .await
                .expect("Failed to send subscription")
        }

        // Spawn custom ping handle (application level ping)
        if let Some(ping_interval) = self.ping_interval.clone() {
            let ping_handler = tokio::spawn(schedule_pings_to_exchange(ws_write, ping_interval));
            _tasks.push(ping_handler);
        }

        let transformer = StatefulTransformer {
            event: vec![Ok(Event::new(12, 12, true, true, 12.0, 12.0))],
        };
        let ws_test = ExchangeStream2::new(ws_stream, transformer);

        Ok(ExchangeStream {
            ws_read: ws_test,
            tasks: _tasks,
        })
    }
}

/*----- */
// Transformer
/*----- */
pub trait Transformer {
    type Error;
    type Input: for<'de> Deserialize<'de>;
    type Output;
    type OutputIter: IntoIterator<Item = Result<Self::Output, Self::Error>>;
    fn transform(&mut self, input: Self::Input) -> Self::OutputIter;
}

#[derive(Clone)]
struct StatefulTransformer {
    event: Vec<Result<Event, SocketError>>,
}

impl Transformer for StatefulTransformer {
    type Error = SocketError;
    type Input = BinanceMessage;
    type Output = Event;
    type OutputIter = Vec<Result<Self::Output, Self::Error>>;

    fn transform(&mut self, input: Self::Input) -> Self::OutputIter {
        // Add new input Trade quantity to sum
        match input {
            BinanceMessage::SubResponse(response) => {}
            BinanceMessage::Book(book_update) => {}
            BinanceMessage::Subscription(sub) => {}
            BinanceMessage::Snapshot(snap) => {}
            BinanceMessage::Trade(trade_update) => {
                // Add new Trade volume to internal state VolumeSum
                let event = Event {
                    timestamp: trade_update.time,
                    seq: trade_update.id,
                    is_trade: true,
                    is_buy: trade_update.side,
                    price: trade_update.price,
                    size: trade_update.amount,
                };
                self.event.push(Ok(event))
            }
        };

        // Return IntoIterator of length 1 containing the running sum of volume
        self.event
    }
}

/*----- */
// Exchange stream
/*----- */
#[pin_project]
pub struct ExchangeStream2<InnerStream, StreamTransformer>
where
    InnerStream: Stream,
    StreamTransformer: Transformer,
{
    #[pin]
    pub stream: InnerStream,
    pub buffer: VecDeque<Result<StreamTransformer::Output, StreamTransformer::Error>>,
    pub transformer: StreamTransformer,
}

impl<InnerStream, StreamTransformer> ExchangeStream2<InnerStream, StreamTransformer>
where
    InnerStream: Stream,
    StreamTransformer: Transformer,
{
    pub fn new(stream: InnerStream, transformer: StreamTransformer) -> Self {
        Self {
            stream,
            transformer,
            buffer: VecDeque::with_capacity(6),
        }
    }
}

impl<InnerStream, StreamTransformer> Stream for ExchangeStream2<InnerStream, StreamTransformer>
where
    InnerStream: Stream<Item = Result<WsMessage, WsError>> + Unpin,
    StreamTransformer: Transformer,
    StreamTransformer::Error: From<SocketError>,
{
    type Item = Result<StreamTransformer::Output, StreamTransformer::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(output) = self.buffer.pop_front() {
                return Poll::Ready(Some(output));
            }

            // Poll inner `Stream` for next the next input protocol message
            let input = match self.as_mut().project().stream.poll_next(cx) {
                Poll::Ready(Some(input)) => input,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };

            let exchange_message = match input {
                Ok(ws_message) => match ws_message {
                    WsMessage::Text(text) => {
                        process_text::<<StreamTransformer as Transformer>::Input>(text)
                    }
                    WsMessage::Binary(binary) => process_binary(binary),
                    WsMessage::Ping(ping) => process_ping(ping),
                    WsMessage::Pong(pong) => process_pong(pong),
                    WsMessage::Close(close_frame) => process_close_frame(close_frame),
                    WsMessage::Frame(frame) => process_frame(frame),
                },
                Err(ws_err) => Some(Err(SocketError::WebSocketError(ws_err))),
            }
            .unwrap()
            .unwrap();

            self.transformer
                .transform(exchange_message)
                .into_iter()
                .for_each(
                    |output_result: Result<StreamTransformer::Output, StreamTransformer::Error>| {
                        self.buffer.push_back(output_result)
                    },
                );
        }
    }
}

pub fn process_text<ExchangeMessage>(
    payload: String,
) -> Option<Result<ExchangeMessage, SocketError>>
where
    ExchangeMessage: DeserializeOwned,
{
    Some(
        serde_json::from_str::<ExchangeMessage>(&payload)
            .map_err(|error| SocketError::Deserialise { error, payload }),
    )
}

pub fn process_binary<ExchangeMessage>(
    payload: Vec<u8>,
) -> Option<Result<ExchangeMessage, SocketError>>
where
    ExchangeMessage: DeserializeOwned,
{
    Some(
        serde_json::from_slice::<ExchangeMessage>(&payload).map_err(|error| {
            SocketError::Deserialise {
                error,
                payload: String::from_utf8(payload).unwrap_or_else(|x| x.to_string()),
            }
        }),
    )
}

pub fn process_ping<ExchangeMessage>(
    ping: Vec<u8>,
) -> Option<Result<ExchangeMessage, SocketError>> {
    format!("{:#?}", ping);
    None
}

pub fn process_pong<ExchangeMessage>(
    pong: Vec<u8>,
) -> Option<Result<ExchangeMessage, SocketError>> {
    format!("{:#?}", pong);
    None
}

pub fn process_close_frame<ExchangeMessage>(
    close_frame: Option<CloseFrame<'_>>,
) -> Option<Result<ExchangeMessage, SocketError>> {
    let close_frame = format!("{:?}", close_frame);
    Some(Err(SocketError::Terminated(close_frame)))
}

pub fn process_frame<ExchangeMessage>(
    frame: Frame,
) -> Option<Result<ExchangeMessage, SocketError>> {
    format!("{:?}", frame);
    None
}
