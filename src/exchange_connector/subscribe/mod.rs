use std::collections::{hash_map, HashMap};

use super::{binance::BinanceSpot, protocols::ws::WsRead, Connector, ExchangeSub, SubGeneric};
use crate::exchange_connector::{
    poloniex::{PoloniexInterface, PoloniexSpot},
    Exchange, FuturesTokio,
};

pub struct ExchangeStream {
    pub stream: WsRead,
    pub tasks: Vec<FuturesTokio>,
}

pub struct StreamBuilder {
    pub streams: HashMap<Exchange, ExchangeStream>,
}

impl Default for StreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamBuilder {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }

    pub fn subscribe(mut self, subscription: Vec<SubGeneric>) -> Self {
        // Convert subscription to exchange specific sub
        let mut exchange_sub = HashMap::new();

        subscription.into_iter().for_each(|sub| {
            exchange_sub
                .entry(sub.exchange)
                .or_insert(Vec::new())
                .push(sub.convert_subscription())
        });

        exchange_sub.into_iter().for_each(|(key, value)| {
            let Some(exchange) = match key {
                Exchange::BinanceSpot => Box::new(&dyn BinanceSpot),
                Exchange::Poloniex => Box::new(PoloniexSpot),
            };
        });

        // exchange_sub.iter().for_each(||)

        println!("{:#?}", exchange_sub);
        self
    }
}
