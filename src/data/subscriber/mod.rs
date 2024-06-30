use std::{
    collections::{HashMap, HashSet},
    ffi::IntoStringError,
};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use super::{
    exchange_connector::{binance::BinanceSpot, poloniex::PoloniexSpot, Connector},
    protocols::ws::{utils::try_connect, WebSocketClient, WsMessage},
    ConnectorStuct, ExchangeSub, StreamType, Sub,
};

/*----- */
// Stream builder
/*----- */
pub struct StreamBuilder<ExchangeConnector> {
    pub clients: HashMap<String, ConnectorStuct<ExchangeConnector>>,
}

impl<ExchangeConnector> Default for StreamBuilder<ExchangeConnector> 
where
    ExchangeConnector: std::marker::Send + Connector + 'static
{
    fn default() -> Self {
        Self::new()
    }
}

impl<ExchangeConnector> StreamBuilder<ExchangeConnector>
where
    ExchangeConnector: std::marker::Send + Connector + 'static
{
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn subscribe<Subscriptions>(mut self, subscriptions: Subscriptions) -> Self
    where
        Subscriptions: IntoIterator<Item = (ExchangeConnector, &'static str, &'static str, StreamType)>,
        ExchangeConnector: Connector + std::cmp::Eq + std::hash::Hash + std::fmt::Debug
    {
        // exchange_sub is a hashmap of where the key is a exchange connector and
        // value is a vec of ExchangeSub structs. Here we are essentially grouping
        // by the exchange connector and geting the subscriptions associated with it
        let mut exchange_sub = HashMap::new();
        subscriptions
            .into_iter()
            .collect::<HashSet<_>>() // rm duplicates
            .into_iter()
            .for_each(|(exchange_conn,base,quote, stream_type)| {
                exchange_sub
                    .entry(exchange_conn)
                    .or_insert(Vec::new())
                    .push(ExchangeSub::new(base, quote, stream_type))
            });

        exchange_sub
            .into_iter()
            .for_each(|(exchange_connector, instruments)| {
                let exchange_id = exchange_connector.exchange_id();
                let subscriptions = ConnectorStuct::new(exchange_connector, instruments);
                self.clients.insert(exchange_id, subscriptions);
            });
        self
    }

    pub async fn init(self) -> HashMap<String, UnboundedReceiver<ExchangeConnector::Output>> {
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
