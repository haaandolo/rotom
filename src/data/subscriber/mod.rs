use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use super::{
    exchange_connector::{binance::BinanceSpot, poloniex::PoloniexSpot, Connector},
    protocols::ws::{utils::try_connect, WebSocketClient, WsMessage},
    Sub,
};
use crate::data::Exchange;

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
                exchange.url(), // Change this to be excconnector
                exchange.requests(&value),
                exchange.ping_interval(),
            );

            self.clients.insert(key, client);
        });
        self
    }

    pub async fn init(self) -> HashMap<Exchange, UnboundedReceiver<String>> {
        self.clients
            .into_iter()
            .map(|(exchange_name, client)| {
                let (tx, rx) = mpsc::unbounded_channel();
                tokio::spawn(try_connect(client, tx));
                (exchange_name, rx)
            })
            .collect::<HashMap<_, _>>()
    }
}
