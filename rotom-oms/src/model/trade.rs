use rotom_data::shared::subscription_models::Instrument;

use super::{OrderId, Side};

#[derive(Debug)]
pub struct PrivateTrade {
    pub id: TradeId,
    pub order_id: OrderId,
    pub instrument: Instrument,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub fees: AssetFees,
}

#[derive(Debug)]
pub struct TradeId(pub String); // can be smolstr

#[derive(Debug)]
pub struct AssetFees(pub f64);
