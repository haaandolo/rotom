use futures::Future;
use std::{collections::HashMap, fmt::Debug, marker::PhantomData, pin::Pin};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{
    data::{
        exchange_connector::{Connector, StreamSelector},
        protocols::ws::connect,
        shared::orderbook::Event,
        ExchangeId, Subscription,
    },
    error::SocketError,
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
    StreamKind: Debug + Send + 'static,
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
        Exchange: Connector + Debug + Send  + StreamSelector<Exchange, StreamKind> + 'static,
    {
        let exchange_sub = subscriptions
            .into_iter()
            .map(Sub::into)
            .collect::<Vec<Subscription<Exchange, StreamKind>>>();

        let (tx, rx) = mpsc::unbounded_channel();
        self.futures.push(Box::pin(async move {
            tokio::spawn(connect::<Exchange, StreamKind>(exchange_sub, tx));
            Ok(())
        }));

        self.market_subscriptions.insert(Exchange::ID, rx);
        self
    }

    pub async fn init(self) -> HashMap<ExchangeId, UnboundedReceiver<Event>> {
        futures::future::try_join_all(self.futures).await.unwrap(); // Chnage to result
        self.market_subscriptions
    }
}
