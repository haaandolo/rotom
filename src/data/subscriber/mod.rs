use std::collections::HashMap;
use std::fmt::Debug;

use multi::MultiStreamBuilder;
use single::StreamBuilder;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamMap};

use super::models::subs::ExchangeId;
use super::models::SubKind;

pub mod multi;
pub mod single;

pub struct Streams<T> {
    pub streams: HashMap<ExchangeId, UnboundedReceiver<T>>,
}

impl<T> Streams<T> {
    pub fn builder<StreamKind>() -> StreamBuilder<StreamKind>
    where
        StreamKind: SubKind + Debug + Send + 'static,
    {
        StreamBuilder::<StreamKind>::new()
    }

    pub fn builder_multi() -> MultiStreamBuilder<T> {
        MultiStreamBuilder::<T>::new()
    }

    pub fn select(&mut self, exchange: ExchangeId) -> Option<mpsc::UnboundedReceiver<T>> {
        self.streams.remove(&exchange)
    }

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

    pub async fn join_map(self) -> StreamMap<ExchangeId, UnboundedReceiverStream<T>> {
        self.streams
            .into_iter()
            .fold(StreamMap::new(), |mut map, (exchange, rx)| {
                map.insert(exchange, UnboundedReceiverStream::new(rx));
                map
            })
    }
}
