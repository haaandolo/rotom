use futures::Future;
use std::{collections::HashMap, fmt::Debug, pin::Pin};
use tokio::sync::mpsc::{self};

use crate::{
    data::{
        exchange::{Connector, StreamSelector},
        models::{
            event::MarketEvent,
            subs::{ExchangeId, Subscription},
            SubKind,
        },
        protocols::ws::connect,
    },
    error::SocketError,
};

use super::Streams;

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
    StreamKind: SubKind + Debug + Send + 'static,
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
        Sub: Into<Subscription<Exchange, StreamKind>> + Debug,
        Exchange: Connector + Debug + Send + StreamSelector<Exchange, StreamKind> + 'static,
    {
        let exchange_sub = subscriptions
            .into_iter()
            .map(Sub::into)
            .collect::<Vec<Subscription<Exchange, StreamKind>>>();

        let exchange_tx = self.channels.entry(Exchange::ID).or_default().tx.clone();

        self.futures.push(Box::pin(async move {
            tokio::spawn(connect::<Exchange, StreamKind>(exchange_sub, exchange_tx));
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
#[derive(Debug)]
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
