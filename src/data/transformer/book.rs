use async_trait::async_trait;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use crate::data::{
    error::SocketError,
    exchange::Identifier,
    model::{event::MarketEvent, event_book::EventOrderBook, subs::Instrument, SubKind},
    shared::orderbook::OrderBook,
};

use super::{ExchangeTransformer, Transformer};

/*----- */
// Map
/*----- */
#[derive(Debug, Default, Clone)]
pub struct Map<T>(pub HashMap<String, T>);

impl<T> Map<T> {
    pub fn find(&self, key: &str) -> Option<&T> {
        self.0.get(key)
    }

    pub fn find_mut(&mut self, key: &str) -> Option<&mut T> {
        self.0.get_mut(key)
    }

    pub fn insert(&mut self, key: String, value: T) -> Option<T> {
        self.0.insert(key, value)
    }
}

/*----- */
// Multi-book transformer
/*----- */
#[derive(Debug, Default)]
pub struct MultiBookTransformer<DeStruct, Updater, StreamKind> {
    pub orderbooks: Map<InstrumentOrderBook<Updater>>,
    marker: PhantomData<(DeStruct, StreamKind)>,
}

/*----- */
// Instrument orderbook
/*----- */
#[derive(Debug, Default)]
pub struct InstrumentOrderBook<Updater> {
    pub instrument: Instrument,
    pub updater: Updater,
    pub book: OrderBook,
}

/*----- */
// Orderbook updater
/*----- */
#[async_trait]
pub trait OrderBookUpdater
where
    Self: Sized + Send,
{
    type OrderBook;
    type UpdateEvent;

    async fn init(instrument: &Instrument) -> Result<InstrumentOrderBook<Self>, SocketError>;

    fn update(
        &mut self,
        book: &mut Self::OrderBook,
        update: Self::UpdateEvent,
    ) -> Result<Option<MarketEvent<EventOrderBook>>, SocketError>;
}

/*----- */
// Impl ExchangeTransformer for MultiBookTransformer
/*----- */
#[async_trait]
impl<Updater, StreamKind> ExchangeTransformer<Updater::UpdateEvent, StreamKind>
    for MultiBookTransformer<Updater::UpdateEvent, Updater, StreamKind>
where
    StreamKind: SubKind<Event = EventOrderBook>,
    Updater: OrderBookUpdater<OrderBook = OrderBook> + Debug,
    Updater::UpdateEvent: Identifier<String> + for<'de> Deserialize<'de>,
{
    async fn new(subs: &[Instrument]) -> Result<Self, SocketError> {
        let init_orderbooks = subs
            .iter()
            .map(|sub| Updater::init(sub))
            .collect::<Vec<_>>();

        let init_orderbooks = futures::future::join_all(init_orderbooks)
            .await
            .into_iter()
            .collect::<Result<Vec<InstrumentOrderBook<Updater>>, SocketError>>()?;

        let book_map = init_orderbooks
            .into_iter()
            .map(|instrument_orderbook| {
                let map_key = format!(
                    "{}{}",
                    &instrument_orderbook.instrument.base, &instrument_orderbook.instrument.quote
                )
                .to_lowercase();
                (map_key, instrument_orderbook)
            })
            .collect::<HashMap<String, InstrumentOrderBook<Updater>>>();

        let orderbooks = Map(book_map);

        Ok(Self {
            orderbooks,
            marker: PhantomData,
        })
    }
}

/*----- */
// Impl Transformer for MultiBookTransformer
/*----- */
impl<Updater, StreamKind> Transformer
    for MultiBookTransformer<Updater::UpdateEvent, Updater, StreamKind>
where
    StreamKind: SubKind<Event = EventOrderBook>,
    Updater: OrderBookUpdater<OrderBook = OrderBook> + Debug,
    Updater::UpdateEvent: Identifier<String> + for<'de> Deserialize<'de>,
{
    type Error = SocketError;
    type Input = Updater::UpdateEvent;
    type Output = MarketEvent<StreamKind::Event>;
    fn transform(&mut self, update: Self::Input) -> Result<Self::Output, Self::Error> {
        let instrument_orderbook =
            self.orderbooks
                .find_mut(&update.id())
                .ok_or(SocketError::OrderBookFindError {
                    symbol: update.id(),
                })?;

        let InstrumentOrderBook {
            instrument: _,
            book,
            updater,
        } = instrument_orderbook;

        match updater.update(book, update) {
            Ok(Some(book)) => Ok(book),
            Ok(None) => Err(SocketError::TransformerNone),
            Err(error) => Err(error),
        }
    }
}
