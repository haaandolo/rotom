use chrono::{DateTime, Utc};
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};

use super::{ClientOrderId, OrderKind};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub enum OrderFill {
    Full,
    Partial,
}

#[derive(Debug)]
pub struct AssetFormatted(pub String);

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct OrderEvent {
    pub time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub client_order_id: ClientOrderId,
    pub instrument: Instrument,
    /// Metadata propagated from source MarketEvent
    pub market_meta: MarketMeta,
    /// LONG, CloseLong, SHORT or CloseShort
    pub decision: Decision,
    /// +ve or -ve Quantity depending on Decision
    pub quantity: f64,
    /// MARKET, LIMIT etc
    pub order_kind: OrderKind,
    pub state: OrderState,
}

impl OrderEvent {
    pub fn get_dollar_value(&self) -> f64 {
        self.quantity * self.market_meta.close
    }
}

impl From<(&ExchangeId, &Instrument)> for AssetFormatted {
    fn from((exchange, instrument): (&ExchangeId, &Instrument)) -> Self {
        match exchange {
            ExchangeId::BinanceSpot => {
                AssetFormatted(format!("{}{}", instrument.base, instrument.quote).to_uppercase())
            }
            ExchangeId::PoloniexSpot => {
                AssetFormatted(format!("{}_{}", instrument.base, instrument.quote).to_uppercase())
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum OrderState {
    Open,
    InTransit,
    Cancelled,
}
