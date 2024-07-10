use futures::Future;
use std::{collections::HashMap, fmt::Debug, marker::PhantomData, pin::Pin};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{data::ExchangeSub, error::SocketError};
use super::{
    exchange_connector::{Connector, StreamSelector},
    protocols::ws::connect,
    shared::orderbook::Event,
    ExchangeId, Subscription,
};

pub type SubscribeFuture = Pin<Box<dyn Future<Output = Result<(), SocketError>>>>;

/*----- */
// Stream builder
/*----- */
#[derive(Default)]
pub struct StreamBuilder<StreamKind> {
    pub market_subscriptions: HashMap<ExchangeId, UnboundedReceiver<Event>>,
    pub futures: Vec<SubscribeFuture>,
    pub exchange_marker: PhantomData<StreamKind>,
}

impl<StreamKind> StreamBuilder<StreamKind>
where
    StreamKind: Debug + Clone + 'static,
{
    pub fn new() -> Self {
        Self {
            market_subscriptions: HashMap::new(),
            futures: Vec::new(),
            exchange_marker: PhantomData,
        }
    }

    pub fn subscribe<SubIter, Sub, Exchange>(mut self, subscriptions: SubIter) -> Self
    where
        SubIter: IntoIterator<Item = Sub>,
        Sub: Into<Subscription<Exchange, StreamKind>> + Debug,
        Exchange: Connector + Debug + Clone + Send + 'static,
        ExchangeSub<Exchange, StreamKind>: StreamSelector<Exchange, StreamKind> + Send,
    {
        let exchange_sub: ExchangeSub<Exchange, StreamKind> = subscriptions
            .into_iter()
            .map(Sub::into)
            .collect::<Vec<_>>()
            .into();

        let exchange_id = exchange_sub.connector.exchange_id();
        let (tx, rx) = mpsc::unbounded_channel();
        self.futures.push(Box::pin(async move {
            tokio::spawn(connect::<Exchange, StreamKind>(exchange_sub, tx));
            Ok(())
        }));

        self.market_subscriptions.insert(exchange_id, rx);
        self
    }

    pub async fn init(self) -> HashMap<ExchangeId, UnboundedReceiver<Event>> {
        futures::future::try_join_all(self.futures).await.unwrap(); // Chnage to result
        self.market_subscriptions
    }
}
