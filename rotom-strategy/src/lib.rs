pub mod spread;

use chrono::{DateTime, Utc};
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument},
    Market, MarketMeta,
};
use std::collections::HashMap;

/*----- */
// Signal Generator
/*----- */
pub trait SignalGenerator {
    fn generate_signal(&mut self, market: &MarketEvent<DataKind>) -> Option<Signal>;
}

/*----- */
// Signal
/*----- */
#[derive(Debug, Clone)]
pub struct Signal {
    pub time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub instrument: Instrument,
    pub signals: HashMap<Decision, SignalStrength>,
    pub market_meta: MarketMeta,
}

#[derive(Debug, Copy, Clone)]
pub struct SignalStrength(pub f64);

/*----- */
// Decision
/*----- */
#[derive(Debug, Default, Eq, PartialEq, PartialOrd, Hash, Clone, Copy)]
pub enum Decision {
    #[default]
    Long,
    CloseLong,
    Short,
    CloseShort,
}

impl Decision {
    pub fn is_long(&self) -> bool {
        matches!(self, Decision::Long)
    }

    pub fn is_short(&self) -> bool {
        matches!(self, Decision::Short)
    }

    pub fn is_entry(&self) -> bool {
        matches!(self, Decision::Short | Decision::Long)
    }

    pub fn is_exit(&self) -> bool {
        matches!(self, Decision::CloseLong | Decision::CloseShort)
    }
}

/*----- */
// Forced signal
/*----- */
#[derive(Debug)]
pub struct SignalForceExit {
    pub time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub instrument: Instrument,
}

impl<M> From<M> for SignalForceExit
where
    M: Into<Market>,
{
    fn from(market: M) -> Self {
        let market = market.into();
        Self::new(market.exchange, market.instrument)
    }
}

impl SignalForceExit {
    pub const FORCED_EXIT_SIGNAL: &'static str = "SignalForcedExit";

    /// Constructs a new [`Self`] using the configuration provided.
    pub fn new<E, I>(exchange: E, instrument: I) -> Self
    where
        E: Into<ExchangeId>,
        I: Into<Instrument>,
    {
        Self {
            time: Utc::now(),
            exchange: exchange.into(),
            instrument: instrument.into(),
        }
    }
}
