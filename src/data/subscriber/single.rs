use futures::Future;
use std::{collections::HashMap, fmt::Debug, pin::Pin};
use tokio::sync::mpsc::{self};

use super::{consume::consume, Streams};
use crate::data::{
    error::SocketError,
    exchange::{self, Connector, StreamSelector},
    model::{
        event::MarketEvent,
        subs::{ExchangeId, Subscription},
        SubKind,
    },
    transformer::{ExchangeTransformer, Transformer},
};

pub type SubscribeFuture = Pin<Box<dyn Future<Output = Result<(), SocketError>>>>;

/*----- */
// Stream builder
/*----- */
#[derive(Default)]
pub struct StreamBuilder<StreamKind>
where
    StreamKind: SubKind,
{
    pub channels: HashMap<ExchangeId, ExchangeChannel<MarketEvent<StreamKind::Event>>>,
    pub futures: Vec<SubscribeFuture>,
}

impl<StreamKind> StreamBuilder<StreamKind>
where
    StreamKind: SubKind + Send + Ord + 'static,
{
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            futures: Vec::new(),
        }
    }

    pub fn subscribe<SubIter, Sub, Exchange>(mut self, subscriptions: SubIter) -> Self
    where
        SubIter: IntoIterator<Item = Sub>,
        Sub: Into<Subscription<Exchange, StreamKind>>,
        Exchange: Connector + Send + StreamSelector<Exchange, StreamKind> + Ord + 'static,
        Exchange::StreamTransformer: ExchangeTransformer<Exchange::Stream, StreamKind> + Debug,
        <Exchange::StreamTransformer as Transformer>::Input: Debug, // DEL
    {
        let mut exchange_sub = subscriptions
            .into_iter()
            .map(Sub::into)
            .collect::<Vec<Subscription<Exchange, StreamKind>>>();

        // Remove duplicates
        exchange_sub.sort();
        exchange_sub.dedup();

        let exchange_tx = self.channels.entry(Exchange::ID).or_default().tx.clone();

        self.futures.push(Box::pin(async move {
            tokio::spawn(consume(exchange_sub, exchange_tx));
            Ok(())
        }));

        self
    }

    pub async fn init(self) -> Result<Streams<MarketEvent<StreamKind::Event>>, SocketError> {
        futures::future::try_join_all(self.futures).await?;
        Ok(Streams {
            streams: self
                .channels
                .into_iter()
                .map(|(exchange, channel)| (exchange, channel.rx))
                .collect(),
        })
    }
}

/*----- */
// Exchange channels
/*----- */
pub struct ExchangeChannel<T> {
    pub tx: mpsc::UnboundedSender<T>,
    pub rx: mpsc::UnboundedReceiver<T>,
}

impl<T> ExchangeChannel<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }
}

impl<T> Default for ExchangeChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}
