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
}

/*----- */
// Spot Balance
/*----- */
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct SpotBalanceId(pub String);

pub fn determine_balance_id(asset: &String, exchange: &ExchangeId) -> SpotBalanceId {
    SpotBalanceId(format!("{}_{}", asset, exchange))
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct AssetBalance {
    pub asset: String, // can be smolstr e.g btc
    pub exchange: ExchangeId,
    pub balance: Balance,
}

impl AssetBalance {
    pub fn new(asset: String, exchange: ExchangeId, balance: Balance) -> Self {
        Self {
            asset,
            exchange,
            balance,
        }
    }
}

impl From<&AssetBalance> for SpotBalanceId {
    fn from(asset_balance: &AssetBalance) -> Self {
        SpotBalanceId(format!("{}_{}", asset_balance.asset, asset_balance.exchange).to_lowercase())
    }
}
