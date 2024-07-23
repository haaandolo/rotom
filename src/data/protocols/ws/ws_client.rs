use std::{
    collections::VecDeque,
    fmt::Debug,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    data::{
        exchange::{Connector, StreamSelector},
        models::{subs::Instrument, SubKind},
        transformer::Transformer,
    },
    error::SocketError,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, Stream, StreamExt,
};
use pin_project::pin_project;
use serde_json::Value;
use tokio::{
    net::TcpStream,
    time::{sleep, Duration},
};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use super::{StreamParser, WebSocketParser};

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

#[derive(Debug)]
#[pin_project]
pub struct ExchangeStream<Exchange, StreamKind, StreamTransformer>
where
    Exchange: Connector + StreamSelector<Exchange, StreamKind> + Send,
    StreamKind: SubKind,
    StreamTransformer: Transformer,
{
    #[pin]
    pub ws_read: WsRead,
    pub transformer: StreamTransformer,
    pub tasks: Vec<JoinHandle>,
    pub buffer: VecDeque<Result<StreamTransformer::Output, StreamTransformer::Error>>,
    pub exchange_marker: PhantomData<(Exchange, StreamKind)>,
}

impl<Exchange, StreamKind, StreamTransformer>
    ExchangeStream<Exchange, StreamKind, StreamTransformer>
where
    Exchange: Connector + StreamSelector<Exchange, StreamKind> + Send,
    StreamKind: SubKind,
    StreamTransformer: Transformer,
{
    pub fn new(stream: WsRead, transformer: StreamTransformer, tasks: Vec<JoinHandle>) -> Self {
        Self {
            ws_read: stream,
            transformer,
            tasks,
            buffer: VecDeque::with_capacity(6),
            exchange_marker: PhantomData,
        }
    }

    pub fn cancel_running_tasks(&self) {
        self.tasks.iter().for_each(|task| {
            task.abort();
        })
    }
}

impl<Exchange, StreamKind, StreamTransformer> Stream
    for ExchangeStream<Exchange, StreamKind, StreamTransformer>
where
    Exchange: Connector + StreamSelector<Exchange, StreamKind> + Send,
    StreamKind: SubKind,
    StreamTransformer: Transformer,
    // StreamTransformer::Error: From<SocketError>,
    StreamTransformer::Output: Debug, //DELETE
{
    type Item = Result<StreamTransformer::Output, StreamTransformer::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Flush Self::Item buffer if it is not currently empty
            if let Some(output) = self.buffer.pop_front() {
                return Poll::Ready(Some(output));
            }

            // Poll inner `Stream` for next the next input protocol message
            let input = match self.as_mut().project().ws_read.poll_next(cx) {
                Poll::Ready(Some(input)) => input,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };

            // Parse input protocol message into `ExchangeMessage`
            let exchange_message = match WebSocketParser::parse::<StreamTransformer::Input>(input) {
                // `StreamParser` successfully deserialised `ExchangeMessage`
                Some(Ok(exchange_message)) => exchange_message,

                // If `StreamParser` returns an Err pass it downstream
                // Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
                Some(Err(err)) => continue,

                // If `StreamParser` returns None it's a safe-to-skip message
                None => continue,
            };

            let trans = self.transformer.transform(exchange_message);

            self.buffer.push_back(Ok(trans))
            /*----- uncomment for later ----- */
            // // Transform `ExchangeMessage` into `Transformer::OutputIter`
            // // ie/ IntoIterator<Item = Result<Output, SocketError>>
            // self.transformer
            //     .transform(exchange_message)
            //     .into_iter()
            //     .for_each(
            //         |output_result: Result<StreamTransformer::Output, StreamTransformer::Error>| {
            //             self.buffer.push_back(output_result)
            //         },
            //     );
        }
    }
}

/*----- */
// WebSocket client
/*----- */
#[derive(Debug)]
pub struct WebSocketClient<Exchange, StreamKind> {
    pub phantom_markers: PhantomData<(Exchange, StreamKind)>,
}

impl<Exchange, StreamKind> WebSocketClient<Exchange, StreamKind>
where
    Exchange: Connector + StreamSelector<Exchange, StreamKind> + Send,
    StreamKind: SubKind,
{
    pub async fn create_websocket(
        subs: &[Instrument],
    ) -> Result<ExchangeStream<Exchange, StreamKind, Exchange::StreamTransformer>, SocketError>
    {
        // Make connection
        let mut tasks = Vec::new();
        let ws = connect_async(Exchange::url().clone())
            .await
            .map(|(ws, _)| ws)
            .map_err(SocketError::WebSocketError);

        // Split WS and make channels
        let (mut ws_write, ws_read) = ws?.split();

        // Handle subscription
        if let Some(subcription) = Exchange::requests(subs).clone() {
            ws_write
                .send(subcription)
                .await
                .expect("Failed to send subscription")
        }

        // Spawn custom ping handle (application level ping)
        if let Some(ping_interval) = Exchange::ping_interval().clone() {
            let ping_handler = tokio::spawn(schedule_pings_to_exchange(ws_write, ping_interval));
            tasks.push(ping_handler);
        }

        let transformer = Exchange::StreamTransformer::default();

        Ok(ExchangeStream::<
            Exchange,
            StreamKind,
            Exchange::StreamTransformer,
        >::new(ws_read, transformer, tasks))
    }
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
