use std::{collections::HashMap, pin::Pin};
use futures::Future;

use crate::{
    data::models::{event::MarketEvent, subs::ExchangeId, SubKind},
    error::SocketError,
};

use super::{
    single::{ExchangeChannel, StreamBuilder},
    Streams,
};

pub type BuilderInitFuture = Pin<Box<dyn Future<Output = Result<(), SocketError>>>>;

#[derive(Default)]
pub struct MultiStreamBuilder<Output> {
    pub channels: HashMap<ExchangeId, ExchangeChannel<Output>>,
    pub futures: Vec<BuilderInitFuture>,
}

impl<Output> MultiStreamBuilder<Output> {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            futures: Vec::new(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add<StreamKind>(mut self, builder: StreamBuilder<StreamKind>) -> Self
    where
        Output: From<MarketEvent<StreamKind::Event>> + Send + 'static,
        StreamKind: SubKind + Send + 'static,
        StreamKind::Event: Send,
    {
        // Allocate HashMap to hold the exchange_tx<Output> for each StreamBuilder exchange present
        let mut exchange_txs = HashMap::with_capacity(builder.channels.len());

        // Iterate over each StreamBuilder exchange present
        for exchange in builder.channels.keys().copied() {
            // Insert ExchangeChannel<Output> Entry to Self for each exchange
            let exchange_tx = self.channels.entry(exchange).or_default().tx.clone();

            // Insert new exchange_tx<Output> into HashMap for each exchange
            exchange_txs.insert(exchange, exchange_tx);
        }

        // Init Streams<Kind::Event> & send mapped Outputs to the associated exchange_tx
        self.futures.push(Box::pin(async move {
            builder
                .init()
                .await?
                .streams
                .into_iter()
                .for_each(|(exchange, mut exchange_rx)| {
                    // Remove exchange_tx<Output> from HashMap that's associated with this tuple:
                    // (ExchangeId, exchange_rx<MarketEvent<SubKind::Event>>)
                    let exchange_tx = exchange_txs
                        .remove(&exchange)
                        .expect("all exchange_txs should be present here");

                    // Task to receive MarketEvent<SubKind::Event> and send Outputs via exchange_tx
                    tokio::spawn(async move {
                        while let Some(event) = exchange_rx.recv().await {
                            let _ = exchange_tx.send(Output::from(event));
                        }
                    });
                });

            Ok(())
        }));

        self
    }

    pub async fn init(self) -> Result<Streams<Output>, SocketError> {
        // Await Stream initialisation perpetual and ensure success
        futures::future::try_join_all(self.futures).await?;

        // Construct Streams<Output> using each ExchangeChannel receiver
        Ok(Streams {
            streams: self
                .channels
                .into_iter()
                .map(|(exchange, channel)| (exchange, channel.rx))
                .collect(),
        })
    }
}
