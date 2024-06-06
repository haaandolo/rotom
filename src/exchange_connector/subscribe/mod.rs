use std::collections::{HashMap, HashSet};

use super::{
    binance::BinanceSpot,
    protocols::ws::{ExchangeStream, WebSocketClient, WebSocketPayload},
    Connector, Sub,
};
use crate::exchange_connector::{poloniex::PoloniexSpot, Exchange};

pub struct StreamBuilder {
    pub streams: HashMap<Exchange, ExchangeStream>,
    pub exchange_sub: Vec<WebSocketPayload>,
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
            exchange_sub: Vec::new(),
        }
    }

    // Function to convert general subscription to exchange specific ones
    pub fn subscribe(mut self, subscriptions: Vec<Sub>) -> Self {
        // Convert vec of subs to hashmap. Extract the exchange
        // field out of the sub struct then put in key of hashmap.
        // Then insert rest of sub fields as value of the hashmap.
        // Vec<Sub> ---> HashMap<Exchange, Vec<ExchangeSub>>
        let mut exchange_sub = HashMap::new();

        subscriptions
            .into_iter()
            .collect::<HashSet<Sub>>() // rm duplicates
            .into_iter()
            .for_each(|sub| {
                exchange_sub
                    .entry(sub.exchange)
                    .or_insert(Vec::new())
                    .push(sub.convert_subscription())
            });

        // Get the connectors for each exchange specified in the subscription
        exchange_sub.into_iter().for_each(|(key, value)| {
            let exchange: Box<&dyn Connector> = match key {
                // Add more connectors here
                Exchange::BinanceSpot => Box::new(&BinanceSpot),
                Exchange::PoloniexSpot => Box::new(&PoloniexSpot),
            };

            self.exchange_sub.push(WebSocketPayload {
                exchange: key,
                url: exchange.url(),
                subscription: exchange.requests(&value),
                ping_interval: exchange.ping_interval(),
            })
        });
        self
    }

    pub async fn init(mut self) -> Self {
        for sub in self.exchange_sub.clone().into_iter() {
            let exchange = sub.exchange;
            let ws = WebSocketClient::connect(sub)
                .await
                .expect("Failed to connect to Ws");
            self.streams.insert(exchange, ws);
        }
        self
    }
}
