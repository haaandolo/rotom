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

#[cfg(test)]
mod test {
    use super::*;

    #[derive(PartialEq, Debug)]
    pub struct Instrument<'a> {
        pub base: &'a str,
        pub quote: &'a str,
        pub stream_type: &'a str,
    }

    impl<'a> Instrument<'a> {
        fn new(_base: &'a str, _quote: &'a str, _stream_type: &'a str) -> Self {
            Self {
                base: _base,
                quote: _quote,
                stream_type: _stream_type,
            }
        }
    }
    #[derive(Eq, PartialEq, Hash, Debug)]
    pub struct BinanceSpot;

    impl BinanceSpot {
        pub fn exchange_id(&self) -> String {
            String::from("BinanceSpot")
        }
    }

    #[derive(PartialEq, Debug)]
    pub struct Subscription<'a, ExchangeConnector> {
        pub connector: ExchangeConnector,
        pub subscriptions: Vec<Instrument<'a>>,
    }

    impl<'a, ExchangeConnector> Subscription<'a, ExchangeConnector> {
        fn new(_connector: ExchangeConnector, _subscriptions: Vec<Instrument<'a>>) -> Self {
            Self {
                connector: _connector,
                subscriptions: _subscriptions,
            }
        }
    }

    #[test]
    fn test_subscribe_function() {
        // This is just an example but should not mix different stream
        // types in the same subscription
        let subscriptions = [
            (BinanceSpot, "arb", "usdt", "Trades"),
            (BinanceSpot, "arb", "usdt", "Trades"),
            (BinanceSpot, "btc", "usdt", "L2"),
        ];

        // Test first part of function
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

        let mut expected_subscriptions = HashMap::new();
        expected_subscriptions.insert(
            BinanceSpot,
            vec![
                Instrument::new("arb", "usdt", "Trades"),
                Instrument::new("btc", "usdt", "L2"),
            ],
        );

        assert_eq!(exchange_subscriptions, exchange_subscriptions);

        // Test second part of function
        let mut market_subscriptions = HashMap::new();
        exchange_subscriptions
            .into_iter()
            .for_each(|(exchange_connector, instruments)| {
                let exchange_id = exchange_connector.exchange_id();
                let subscriptions = Subscription::new(exchange_connector, instruments);
                market_subscriptions.insert(exchange_id, subscriptions);
            });

        let mut expected_market_subscriptions = HashMap::new();
        expected_market_subscriptions.insert(
            "BinanceSpot".to_string(),
            Subscription::new(
                BinanceSpot,
                vec![
                    Instrument::new("arb", "usdt", "Trades"),
                    Instrument::new("btc", "usdt", "L2"),
                ],
            ),
        );

        assert_eq!(market_subscriptions, expected_market_subscriptions);
    }
}
