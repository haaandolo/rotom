use std::{collections::HashMap, marker::PhantomData};

use async_trait::async_trait;

use crate::{data::models::subs::Instrument, error::SocketError};

// Books on exchange likely has different sequencing and initialisation methods
// 1. Have generalised orderbook but have exchange specific sequencer and init methods

pub struct MultiBookTransformer<Input, Book> {
    pub orderbooks: HashMap<String, Book>,
    marker: PhantomData<Input>,
}

#[async_trait]
pub trait OrderBookUpdater
where
    Self: Sized,
{
    type OrderBook;
    type UpdateEvent;

    async fn init<Exchange, StreamKind>(instrument: Instrument) {}

    async fn update(
        &mut self,
        book: &mut Self::OrderBook,
        update: Self::UpdateEvent,
    ) -> Result<Self::OrderBook, SocketError>;
}
