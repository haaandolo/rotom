use futures::Future;
use serde::Deserialize;
use std::{
    collections::{HashMap},
    pin::Pin,
};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{
    data::{
        exchange_connector::Connector, protocols::ws::connect, ExchangeId, Instrument, StreamType,
        Subscription,
    },
    error::SocketError,
};

pub type SubscribeFuture = Pin<Box<dyn Future<Output = Result<(), SocketError>>>>;

/*----- */
// Stream builder
/*----- */
pub struct StreamBuilder<StreamKind> {
    pub market_subscriptions: HashMap<ExchangeId, UnboundedReceiver<StreamKind>>,
    pub futures: Vec<SubscribeFuture>,
}

impl<StreamKind> Default for StreamBuilder<StreamKind>
where
    StreamKind: Send + for<'de> Deserialize<'de> + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<StreamKind> StreamBuilder<StreamKind>
where
    StreamKind: Send + for<'de> Deserialize<'de> + 'static,
{
    pub fn new() -> Self {
        Self {
            market_subscriptions: HashMap::new(),
            futures: Vec::new(),
        }
    }

    pub fn subscribe<Subscriptions, ExchangeConnector>(
        mut self,
        subscriptions: Subscriptions,
    ) -> Self
    where
        Subscriptions: IntoIterator<
            Item = (
                ExchangeConnector,
                &'static str,
                &'static str,
                StreamType,
                StreamKind,
            ),
        >,
        ExchangeConnector: Connector + Eq + Send + 'static + std::hash::Hash,
    {
        let mut exchange_subscriptions = HashMap::new();
        subscriptions
            .into_iter()
            //            .collect::<HashSet<_>>() // rm duplicates
            //            .into_iter()
            .for_each(|(exchange_conn, base, quote, stream_type, Kind)| {
                exchange_subscriptions
                    .entry(exchange_conn)
                    .or_insert(Vec::new())
                    .push(Instrument::new(base, quote, stream_type))
            });

        exchange_subscriptions
            .into_iter()
            .for_each(|(exchange_connector, instruments)| {
                let exchange_id = exchange_connector.exchange_id();
                let subscriptions = Subscription::new(exchange_connector, instruments);
                let (tx, rx) = mpsc::unbounded_channel();
                self.futures.push(Box::pin(async move {
                    tokio::spawn(connect::<StreamKind, ExchangeConnector>(subscriptions, tx));
                    Ok(())
                }));
                self.market_subscriptions.insert(exchange_id, rx);
            });
        self
    }

    pub async fn init(self) -> HashMap<ExchangeId, UnboundedReceiver<StreamKind>> {
        futures::future::try_join_all(self.futures).await.unwrap(); // Chnage to result
        self.market_subscriptions
    }
}
