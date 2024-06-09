use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use super::{
    binance::BinanceSpot,
    protocols::ws::{try_connect, WebSocketClient, WsMessage},
    Connector, Sub,
};
use crate::data::{poloniex::PoloniexSpot, Exchange};

/*---------- */
// Stream builder
/*---------- */
pub struct StreamBuilder {
    pub clients: HashMap<Exchange, WebSocketClient>,
}

impl Default for StreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamBuilder {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
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

            let client = WebSocketClient::new(
                exchange.url(),
                exchange.requests(&value),
                exchange.ping_interval(),
            );

            self.clients.insert(key, client);
        });
        self
    }

    pub async fn init(self) -> HashMap<Exchange, UnboundedReceiver<WsMessage>> {
        let mut ws_streams = HashMap::new();
        for (key, client) in self.clients.into_iter() {
            let (tx, rx) = mpsc::unbounded_channel();
            tokio::spawn(try_connect(client, tx));
            ws_streams.insert(key, rx);
        }
        ws_streams
    }
}
