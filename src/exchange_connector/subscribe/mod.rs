use std::collections::HashMap;

use super::{
    binance::BinanceSpot,
    protocols::ws::{WebSocketBuilder, WsRead},
    Connector, Sub,
};
use crate::exchange_connector::{poloniex::PoloniexSpot, Exchange, FuturesTokio};

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

    pub async fn subscribe(mut self, subscriptions: Vec<Sub>) -> Self {
        // Convert subscription to exchange specific subscription
        let mut exchange_sub = HashMap::new();
        for sub in subscriptions.into_iter() {
            exchange_sub
                .entry(sub.exchange)
                .or_insert(Vec::new())
                .push(sub.convert_subscription())
        }

        // Get the connectors for each exchange specified in the subscription
        for (key, value) in exchange_sub.into_iter() {
            let exchange: Box<&dyn Connector> = match key {
                Exchange::BinanceSpot => Box::new(&BinanceSpot),
                Exchange::Poloniex => Box::new(&PoloniexSpot),
            };

            let url = exchange.url();
            let subscription = exchange.requests(&value);
            let ping_interval = exchange.ping_interval();

            let ws = WebSocketBuilder::new(url)
                .set_subscription(subscription)
                .set_ping_interval(ping_interval)
                .build()
                .await
                .unwrap();

            self.streams.insert(key, ws);
        }

        self
    }
}
