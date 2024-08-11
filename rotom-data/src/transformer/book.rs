use async_trait::async_trait;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use crate::{
    assets::orderbook::OrderBook,
    error::SocketError,
    event_models::{event_book::EventOrderBook, market_event::MarketEvent, SubKind},
    exchange::{Connector, Identifier},
    shared::subscription_models::{ExchangeSubscription, Instrument},
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
impl<Exchange, Updater, StreamKind> ExchangeTransformer<Exchange, Updater::UpdateEvent, StreamKind>
    for MultiBookTransformer<Updater::UpdateEvent, Updater, StreamKind>
where
    Exchange: Connector + Sync,
    Exchange::Market: AsRef<str>,
    StreamKind: SubKind<Event = EventOrderBook>,
    Updater: OrderBookUpdater<OrderBook = OrderBook> + Debug,
    Updater::UpdateEvent: Identifier<String> + for<'de> Deserialize<'de>,
{
    async fn new(
        subs: &[ExchangeSubscription<Exchange, Exchange::Channel, Exchange::Market>],
    ) -> Result<Self, SocketError> {
        let (symbols, init_orderbooks): (Vec<_>, Vec<_>) = subs
            .iter()
            .map(|sub| {
                (
                    String::from(sub.market.as_ref()),
                    Updater::init(&sub.instrument),
                )
            })
            .unzip();

        let init_orderbooks = futures::future::join_all(init_orderbooks)
            .await
            .into_iter()
            .collect::<Result<Vec<InstrumentOrderBook<Updater>>, SocketError>>()?;

        let book_map = symbols
            .into_iter()
            .zip(init_orderbooks.into_iter())
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
