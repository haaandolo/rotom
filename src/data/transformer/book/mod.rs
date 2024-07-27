use async_trait::async_trait;
use std::{collections::HashMap, marker::PhantomData};

use crate::{
    data::{model::subs::Instrument, shared::orderbook::OrderBook},
    error::SocketError,
};

// Books on exchange likely has different sequencing and initialisation methods
// 1. Have generalised orderbook but have exchange specific sequencer and init methods

/*----- */
// Multi-book transformer
/*----- */
#[derive(Debug, Default, Clone)]
pub struct Map<T>(pub HashMap<String, T>);

#[derive(Debug, Default)]
pub struct MultiBookTransformer<Input, InstrumentId, Updater> {
    pub orderbooks: Map<InstrumentOrderBook<InstrumentId, Updater>>,
    marker: PhantomData<Input>,
}

/*----- */
// Instrument orderbook
/*----- */
#[derive(Debug, Default)]
pub struct InstrumentOrderBook<InstrumentId, Updater> {
    pub instrument: InstrumentId,
    pub updater: Updater,
    pub book: OrderBook,
}

/*----- */
// Orderbook updater
/*----- */
#[async_trait]
pub trait OrderBookUpdater
where
    Self: Sized,
{
    type OrderBook;
    type UpdateEvent;

    async fn init<Exchange, StreamKind>(
        instrument: Instrument,
    ) -> Result<InstrumentOrderBook<Instrument, Self>, SocketError>;

    async fn update(
        &mut self,
        book: &mut Self::OrderBook,
        update: Self::UpdateEvent,
    ) -> Result<Option<Self::OrderBook>, SocketError>;
}
