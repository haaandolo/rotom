use chrono::{DateTime, Utc};
use rotom_data::shared::subscription_models::{ExchangeId, Instrument};

pub mod error;
pub mod live;

pub trait MarketGenerator<Event> {
    fn next(&mut self) -> Feed<Event>;
}

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
            close: 100.0,
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
