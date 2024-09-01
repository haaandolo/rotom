pub mod allocator;
pub mod error;
pub mod portfolio;
pub mod position;
pub mod repository;
pub mod risk;

use chrono::{DateTime, Utc};
use error::PortfolioError;
use position::PositionUpdate;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument}, MarketMeta,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    event::Event,
    execution::FillEvent,
    strategy::{Decision, Signal, SignalForceExit},
};

/*----- */
// Market Updater
/*----- */
pub trait MarketUpdater {
    fn update_from_market(
        &mut self,
        market: &MarketEvent<DataKind>,
    ) -> Result<Option<PositionUpdate>, PortfolioError>;
}

/*----- */
// Order Generator
/*----- */
pub trait OrderGenerator {
    fn generate_order(&mut self, signal: &Signal) -> Result<Option<OrderEvent>, PortfolioError>;

    fn generate_exit_order(
        &mut self,
        signal: SignalForceExit,
    ) -> Result<Option<OrderEvent>, PortfolioError>;
}

/*----- */
// Fill Updater
/*----- */
pub trait FillUpdater {
    fn update_from_fill(&mut self, fill: &FillEvent) -> Result<Vec<Event>, PortfolioError>;
}

/*----- */
// Order Event
/*----- */
#[derive(Debug, Clone)]
pub struct OrderEvent {
    pub time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub instrument: Instrument,
    pub market_meta: MarketMeta,
    pub decision: Decision,
    pub quantity: f64,
    pub order_type: OrderType,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
}

impl Default for OrderType {
   fn default() -> Self {
       Self::Market
   } 
}

/*----- */
// Order Event Builder
/*----- */
#[derive(Debug, Default)]
pub struct OrderEventBuilder {
    pub time: Option<DateTime<Utc>>,
    pub exchange: Option<ExchangeId>,
    pub instrument: Option<Instrument>,
    pub market_meta: Option<MarketMeta>,
    pub decision: Option<Decision>,
    pub quantity: Option<f64>,
    pub order_type: Option<OrderType>,
}

impl OrderEventBuilder {
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

    pub fn order_type(self, value: OrderType) -> Self {
        Self {
            order_type: Some(value),
            ..self
        }
    }

    pub fn build(self) -> Result<OrderEvent, PortfolioError> {
        Ok(OrderEvent {
            time: self.time.ok_or(PortfolioError::BuilderIncomplete("time"))?,
            exchange: self
                .exchange
                .ok_or(PortfolioError::BuilderIncomplete("exchange"))?,
            instrument: self
                .instrument
                .ok_or(PortfolioError::BuilderIncomplete("instrument"))?,
            market_meta: self
                .market_meta
                .ok_or(PortfolioError::BuilderIncomplete("market_meta"))?,
            decision: self
                .decision
                .ok_or(PortfolioError::BuilderIncomplete("decision"))?,
            quantity: self
                .quantity
                .ok_or(PortfolioError::BuilderIncomplete("quantity"))?,
            order_type: self
                .order_type
                .ok_or(PortfolioError::BuilderIncomplete("order_type"))?,
        })
    }

}

/*----- */
// Balance
/*----- */
pub type BalanceId = String;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Balance {
    pub time: DateTime<Utc>,
    pub total: f64,
    pub available: f64
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            time: Utc::now(),
            total: 0.0,
            available: 0.0,
        }
    }
}

impl Balance {
    pub fn new(time: DateTime<Utc>, total: f64, available: f64) -> Self {
        Self {
            time,
            total,
            available,
        }
    }

    pub fn balance_id(engine_id: Uuid) -> BalanceId {
        format!("{}_balance", engine_id)
    }
}