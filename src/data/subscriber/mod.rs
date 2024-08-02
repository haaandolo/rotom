pub mod multi;
pub mod single;
pub mod consume;

use std::collections::HashMap;
use single::StreamBuilder;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamMap};

use multi::MultiStreamBuilder;
use super::model::subs::ExchangeId;
use super::model::SubKind;

pub struct Streams<T> {
    pub streams: HashMap<ExchangeId, UnboundedReceiver<T>>,
}

impl<T> Streams<T> {
    // Single StreamKind builder
    pub fn builder<StreamKind>() -> StreamBuilder<StreamKind>
    where
        StreamKind: SubKind + Send + Ord + 'static,
    {
        StreamBuilder::<StreamKind>::new()
    }

    // Multi StreamKind builder
    pub fn builder_multi() -> MultiStreamBuilder<T> {
        MultiStreamBuilder::<T>::new()
    }

    // Remove exchange from hashmap
    pub fn select(&mut self, exchange: ExchangeId) -> Option<mpsc::UnboundedReceiver<T>> {
        self.streams.remove(&exchange)
    }

    // Join all exchange streams into a unified mpsc::UnboundedReceiver
    pub async fn join(self) -> mpsc::UnboundedReceiver<T>
    where
        T: Send + 'static,
    {
        let (joined_tx, joined_rx) = mpsc::unbounded_channel();

        for mut exchange_rx in self.streams.into_values() {
            let joined_tx = joined_tx.clone();
            tokio::spawn(async move {
                while let Some(event) = exchange_rx.recv().await {
                    let _ = joined_tx.send(event);
                }
            });
        }

        joined_rx
    }

    // Join all exchange mpsc::UnboundedReceiver streams into a unified StreamMap
    pub async fn join_map(self) -> StreamMap<ExchangeId, UnboundedReceiverStream<T>> {
        self.streams
            .into_iter()
            .fold(StreamMap::new(), |mut map, (exchange, rx)| {
                map.insert(exchange, UnboundedReceiverStream::new(rx));
                map
            })
    }
}
