pub mod assets;
pub mod error;
pub mod event_models;
pub mod exchange;
pub mod metric;
pub mod protocols;
pub mod shared;
pub mod streams;
pub mod transformer;

use chrono::{DateTime, Utc};
use shared::subscription_models::{ExchangeId, Instrument};
use tokio::sync::mpsc::{self, UnboundedReceiver};

/*----- */
// MarketGenerator
/*----- */
pub trait MarketGenerator<Event> {
    fn next(&mut self) -> Feed<Event>;
}

/*----- */
// Feed
/*----- */
#[derive(Debug)]
pub enum Feed<Event> {
    Next(Event),
    UnHealthy,
    Finished,
}

/*----- */
// Market metadata
/*----- */
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MarketMeta {
    pub close: f64,
    pub time: DateTime<Utc>,
}

impl Default for MarketMeta {
    fn default() -> Self {
        Self {
            close: 50000.0,
            time: Utc::now(),
        }
    }
}

/*----- */
// Markets
/*----- */
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Market {
    pub exchange: ExchangeId,
    pub instrument: Instrument,
}

impl Market {
    pub fn new(exchange: ExchangeId, instrument: Instrument) -> Self {
        Self {
            exchange,
            instrument,
        }
    }
}

impl From<(ExchangeId, Instrument)> for Market {
    fn from((exchange, instrument): (ExchangeId, Instrument)) -> Self {
        Self {
            exchange,
            instrument,
        }
    }
}

/*----- */
// Market ID
/*----- */
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct MarketId(pub String);

impl MarketId {
    pub fn new(exchange: &ExchangeId, instrument: &Instrument) -> MarketId {
        MarketId(format!(
            "{}_{}_{}",
            exchange, instrument.base, instrument.quote
        ))
    }
}

impl From<&Market> for MarketId {
    fn from(market: &Market) -> MarketId {
        MarketId(format!(
            "{}_{}_{}",
            market.exchange, market.instrument.base, market.instrument.quote
        ))
    }
}

/*----- */
// Market feed
/*----- */
#[derive(Debug)]
pub struct MarketFeed<Event> {
    pub market_rx: UnboundedReceiver<Event>,
}

impl<Event> MarketFeed<Event> {
    pub fn new(market_rx: UnboundedReceiver<Event>) -> Self {
        Self { market_rx }
    }
}

/*----- */
// Impl MarketGenerator
/*----- */
impl<Event> MarketGenerator<Event> for MarketFeed<Event> {
    fn next(&mut self) -> Feed<Event> {
        loop {
            match self.market_rx.try_recv() {
                Ok(event) => break Feed::Next(event),
                Err(mpsc::error::TryRecvError::Empty) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => break Feed::Finished,
            }
        }
    }
}
