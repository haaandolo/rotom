use futures::Stream;
use pin_project::pin_project;
use std::{
    collections::VecDeque,
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};

use super::{
    ws_parser::{StreamParser, WebSocketParser},
    JoinHandle, WsRead,
};
use crate::data::{error::SocketError, transformer::Transformer};

/*----- */
// Exchange Stream
/*----- */
#[derive(Debug)]
#[pin_project]
pub struct ExchangeStream<StreamTransformer>
where
    StreamTransformer: Transformer,
{
    #[pin]
    pub ws_read: WsRead,
    pub transformer: StreamTransformer,
    pub tasks: Vec<JoinHandle>,
    pub buffer: VecDeque<Result<StreamTransformer::Output, StreamTransformer::Error>>,
}

impl<StreamTransformer> ExchangeStream<StreamTransformer>
where
    StreamTransformer: Transformer,
{
    pub fn new(stream: WsRead, transformer: StreamTransformer, tasks: Vec<JoinHandle>) -> Self {
        Self {
            ws_read: stream,
            transformer,
            tasks,
            buffer: VecDeque::with_capacity(6),
        }
    }

    pub fn cancel_running_tasks(&self) {
        self.tasks.iter().for_each(|task| {
            task.abort();
        })
    }
}

/*----- */
// Poll next implementation
/*----- */
impl<StreamTransformer> Stream for ExchangeStream<StreamTransformer>
where
    StreamTransformer: Transformer,
    StreamTransformer::Error: From<SocketError>,
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
                Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),

                // If `StreamParser` returns None it's a safe-to-skip message
                None => continue,
            };

            let transformed_message = self.transformer.transform(exchange_message);
            self.buffer.push_back(Ok(transformed_message?))
        }
    }
}
