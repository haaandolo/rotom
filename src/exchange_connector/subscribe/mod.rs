use std::collections::{HashMap, HashSet};

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

        println!("{:#?}", exchange_sub);

        // Get the connectors for each exchange specified in the subscription
        for (key, value) in exchange_sub.into_iter() {
            let exchange: Box<&dyn Connector> = match key {
                // Add more connectors here
                Exchange::BinanceSpot => Box::new(&BinanceSpot),
                Exchange::PoloniexSpot => Box::new(&PoloniexSpot),
            };

            println!("rm dup");
            let unique_vec: Vec<_> = value
                .clone()
                .into_iter()
                .collect::<HashSet<_>>() // Convert to HashSet to remove duplicates
                .into_iter()
                .collect();
            
            println!("{:#?}", unique_vec);

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
