pub mod spread;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
};

/*----- */
// Signal Generator
/*----- */
pub trait SignalGenerator {
    fn generate_signal(&mut self, market: &MarketEvent<DataKind>) -> Option<Signal>;
}

/*----- */
// Signal
/*----- */
pub struct Signal {
    pub time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub instrument: String,
    pub signals: HashMap<Decision, SignalStrength>,
}

#[derive(Debug)]
pub struct SignalStrength(pub f64);

/*----- */
// Decision
/*----- */
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Decision {
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
