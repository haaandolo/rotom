pub mod error;
pub mod simulated;

use chrono::{DateTime, Utc};
use error::ExecutionError;
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};

use crate::portfolio::OrderEvent;

/*----- */
// Execution Client
/*----- */
pub trait ExecutionClient {
    fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError>;
}

/*----- */
// Fill Event
/*----- */
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct FillEvent {
    pub time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub instrument: Instrument,
    pub market_meta: MarketMeta,
    pub decision: Decision,
    pub quantity: f64,
    pub fill_value_gross: f64,
    pub fees: Fees,
}

impl FillEvent {
    pub const EVENT_TYPE: &'static str = "Fill";

    pub fn builder() -> FillEventBuilder {
        FillEventBuilder::new()
    }
}

/*----- */
// Fees
/*----- */
#[derive(Default, Debug, PartialEq, PartialOrd, Clone, Deserialize, Serialize, Copy)]
pub struct Fees {
    pub exchange: FeeAmount,
    pub slippage: FeeAmount,
    pub network: FeeAmount,
}

impl Fees {
    pub fn calculate_total_fees(&self) -> f64 {
        self.exchange + self.network + self.slippage
    }
}

pub type FeeAmount = f64;
/*----- */
// Fill Event Builder
/*----- */
#[derive(Debug, Default)]
pub struct FillEventBuilder {
    pub time: Option<DateTime<Utc>>,
    pub exchange: Option<ExchangeId>,
    pub instrument: Option<Instrument>,
    pub market_meta: Option<MarketMeta>,
    pub decision: Option<Decision>,
    pub quantity: Option<f64>,
    pub fill_value_gross: Option<f64>,
    pub fees: Option<Fees>,
}

impl FillEventBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn time(self, value: DateTime<Utc>) -> Self {
        Self {
            time: Some(value),
            ..self
        }
    }

    pub fn exchange(self, value: ExchangeId) -> Self {
        Self {
            exchange: Some(value),
            ..self
        }
    }

    pub fn instrument(self, value: Instrument) -> Self {
        Self {
            instrument: Some(value),
            ..self
        }
    }

    pub fn market_meta(self, value: MarketMeta) -> Self {
        Self {
            market_meta: Some(value),
            ..self
        }
    }

    pub fn decision(self, value: Decision) -> Self {
        Self {
            decision: Some(value),
            ..self
        }
    }

    pub fn quantity(self, value: f64) -> Self {
        Self {
            quantity: Some(value),
            ..self
        }
    }

    pub fn fill_value_gross(self, value: f64) -> Self {
        Self {
            fill_value_gross: Some(value),
            ..self
        }
    }

    pub fn fees(self, value: Fees) -> Self {
        Self {
            fees: Some(value),
            ..self
        }
    }

    pub fn build(self) -> Result<FillEvent, ExecutionError> {
        Ok(FillEvent {
            time: self.time.ok_or(ExecutionError::BuilderIncomplete("time"))?,
            exchange: self
                .exchange
                .ok_or(ExecutionError::BuilderIncomplete("exchange"))?,
            instrument: self
                .instrument
                .ok_or(ExecutionError::BuilderIncomplete("instrument"))?,
            market_meta: self
                .market_meta
                .ok_or(ExecutionError::BuilderIncomplete("market meta"))?,
            decision: self
                .decision
                .ok_or(ExecutionError::BuilderIncomplete("decision"))?,
            quantity: self
                .quantity
                .ok_or(ExecutionError::BuilderIncomplete("quantity"))?,
            fill_value_gross: self
                .fill_value_gross
                .ok_or(ExecutionError::BuilderIncomplete("fill_gross_value"))?,
            fees: self.fees.ok_or(ExecutionError::BuilderIncomplete("fees"))?,
        })
    }
}
