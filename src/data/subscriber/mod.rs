use futures::Future;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    pin::Pin,
};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{data::ExchangeSub, error::SocketError};

use super::{
    exchange_connector::Connector, protocols::ws::connect, shared::orderbook::Event, ExchangeId,
    Instrument, StreamType, Subscription, Subscription2,
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

impl<StreamKind> StreamBuilder<StreamKind> {
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
        Sub: Into<Subscription2<Exchange, StreamKind>> + std::fmt::Debug,
        Exchange: Connector + std::fmt::Debug + Clone,
        StreamKind: std::fmt::Debug + Clone, // StreamKind: Send + for<'de> Deserialize<'de> + 'static + Into<Event> + std::fmt::Debug,
    {
        let subscriptions: ExchangeSub<Exchange, StreamKind> = subscriptions
            .into_iter()
            .map(Sub::into)
            .collect::<Vec<_>>()
            .into();

        self.futures.push(Box::pin(async move {
            tokio::spawn(connect::<Exchange, StreamKind>(subscriptions, tx));
            Ok(())
        }));

        println!("{:#?}", subscriptions);

        // let mut exchange_subscriptions = HashMap::new();
        // subscriptions
        //     .into_iter()
        //     // .collect::<HashSet<_>>() // rm duplicates
        //     // .into_iter()
        //     .for_each(|(exchange_conn, base, quote, stream_type, kind)| {
        //         exchange_subscriptions
        //             .entry(exchange_conn)
        //             .or_insert(Vec::new())
        //             .push(Instrument::new(base, quote, stream_type))
        //     });

        // exchange_subscriptions
        //     .into_iter()
        //     .for_each(|(exchange_connector, instruments)| {
        //         let exchange_id = exchange_connector.exchange_id();
        //         let subscriptions = Subscription::new(exchange_connector, instruments);
        //         let (tx, rx) = mpsc::unbounded_channel();
        //         self.futures.push(Box::pin(async move {
        //             tokio::spawn(connect::<ExchangeConnector, StreamKind>(subscriptions, tx));
        //             Ok(())
        //         }));
        //         self.market_subscriptions.insert(exchange_id, rx);
        //     });
        self
    }

    // pub fn subscribe<StreamKind>(
    //     mut self,
    //     subscriptions: impl IntoIterator<
    //         Item = (ExchangeConnector, &'static str, &'static str, StreamType, StreamKind),
    //     >,
    // ) -> Self
    // where
    //     ExchangeConnector: Connector + Eq + std::hash::Hash + std::fmt::Debug,
    //     StreamKind: Send + for<'de> Deserialize<'de> + 'static + Into<Event>,
    // {
    //     let mut exchange_subscriptions = HashMap::new();
    //     subscriptions
    //         .into_iter()
    //         // .collect::<HashSet<_>>() // rm duplicates
    //         // .into_iter()
    //         .for_each(|(exchange_conn, base, quote, stream_type, kind)| {
    //             exchange_subscriptions
    //                 .entry(exchange_conn)
    //                 .or_insert(Vec::new())
    //                 .push(Instrument::new(base, quote, stream_type))
    //         });

    //     exchange_subscriptions
    //         .into_iter()
    //         .for_each(|(exchange_connector, instruments)| {
    //             let exchange_id = exchange_connector.exchange_id();
    //             let subscriptions = Subscription::new(exchange_connector, instruments);
    //             let (tx, rx) = mpsc::unbounded_channel();
    //             self.futures.push(Box::pin(async move {
    //                 tokio::spawn(connect::<ExchangeConnector, StreamKind>(subscriptions, tx));
    //                 Ok(())
    //             }));
    //             self.market_subscriptions.insert(exchange_id, rx);
    //         });
    //     self
    // }

    pub async fn init(self) -> HashMap<ExchangeId, UnboundedReceiver<Event>> {
        futures::future::try_join_all(self.futures).await.unwrap(); // Chnage to result
        self.market_subscriptions
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
