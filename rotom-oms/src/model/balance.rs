use rotom_data::shared::subscription_models::ExchangeId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/*----- */
// Balance
/*----- */
pub type BalanceId = String;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Balance {
    pub total: f64,
    pub available: f64, // only used for margin will be zero if spot
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            total: 0.0,
            available: 0.0,
        }
    }
}

impl Balance {
    pub fn new(total: f64, available: f64) -> Self {
        Self { total, available }
    }

    pub fn balance_id(engine_id: Uuid) -> BalanceId {
        format!("{}_balance", engine_id)
    }

    pub fn apply_delta(&mut self, delta: BalanceDelta) {
        self.total += delta.total;
        self.available += delta.available;
    }
}

// Used for when deposits and withdrawls happen and shows the delta
#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct BalanceDelta {
    pub asset: String,
    pub exchange: ExchangeId,
    pub total: f64,
    pub available: f64, // only used for margin will be zero if spot
}

/*----- */
// Spot Balance id
/*----- */
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct SpotBalanceId(pub String);

pub fn determine_balance_id(asset: &String, exchange: &ExchangeId) -> SpotBalanceId {
    SpotBalanceId(format!("{}_{}", asset, exchange))
}
