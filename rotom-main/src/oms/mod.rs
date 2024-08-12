pub mod allocator;
pub mod error;
pub mod portfolio;
pub mod position;
pub mod repository;
pub mod risk;

use chrono::{DateTime, Utc};
use rotom_data::shared::subscription_models::ExchangeId;

use crate::{data::MarketMeta, strategy::Decision};

#[derive(Debug)]
pub struct OrderEvent {
    pub time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub instrument: String,
    pub market_meta: MarketMeta,
    pub decision: Decision,
    pub quantity: f64,
    pub order_type: OrderType,
}

#[derive(Debug)]
pub enum OrderType {
    Market,
    Limit,
}
