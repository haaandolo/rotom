use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use super::{
    exchange_connector::Connector, protocols::ws::connect, Instrument, StreamType, Subscription,
};

/*----- */
// Stream builder
/*----- */
pub struct StreamBuilder<ExchangeConnector> {
    pub market_subscriptions: HashMap<String, Subscription<ExchangeConnector>>,
}

impl<ExchangeConnector> Default for StreamBuilder<ExchangeConnector>
where
    ExchangeConnector: Send + Connector + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<ExchangeConnector> StreamBuilder<ExchangeConnector>
where
    ExchangeConnector: Send + Connector + 'static,
{
    pub fn new() -> Self {
        Self {
            market_subscriptions: HashMap::new(),
        }
    }

    pub fn subscribe<Subscriptions>(mut self, subscriptions: Subscriptions) -> Self
    where
        Subscriptions:
            IntoIterator<Item = (ExchangeConnector, &'static str, &'static str, StreamType)>,
        ExchangeConnector: Connector + Eq + std::hash::Hash + std::fmt::Debug,
    {
        // exchange_sub is a hashmap of where the key is a exchange connector and
        // value is a vec of ExchangeSub structs. Here we are essentially grouping
        // by the exchange connector and geting the subscriptions associated with it
        let mut exchange_subscriptions = HashMap::new();
        subscriptions
            .into_iter()
            .collect::<HashSet<_>>() // rm duplicates
            .into_iter()
            .for_each(|(exchange_conn, base, quote, stream_type)| {
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
                self.market_subscriptions.insert(exchange_id, subscriptions);
            });
        self
    }

    pub async fn init(self) -> HashMap<String, UnboundedReceiver<ExchangeConnector::Output>> {
        self.market_subscriptions
            .into_iter()
            .map(|(exchange_name, subscription)| {
                let (tx, rx) = mpsc::unbounded_channel();
                tokio::spawn(connect(subscription, tx));
                (exchange_name, rx)
            })
            .collect::<HashMap<_, _>>()
    }
}
