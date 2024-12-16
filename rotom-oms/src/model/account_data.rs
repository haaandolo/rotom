use chrono::{DateTime, Utc};
use rotom_data::shared::subscription_models::ExchangeId;
use serde::Deserialize;

use super::{
    balance::{Balance, SpotBalanceId},
    Side,
};

/*----- */
// State
/*----- */
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum OrderStatus {
    New,
    Canceled,
    Rejected,
    Expired,
    PendingNew,
    PartiallyFilled,
    Filled,
    Trade,
    PendingCancel,
    ExpiredInMatch,
    PartiallyCanceled,
    Failed,
}

#[derive(Debug)]
pub struct AccountDataOrder {
    pub exchange: ExchangeId,
    pub client_order_id: String,
    pub asset: String,
    pub price: f64,    // todo: check if absolute or incremental
    pub quantity: f64, // todo: check if absolute or incremental
    pub status: OrderStatus,
    pub execution_time: DateTime<Utc>,
    pub side: Side,
    pub fee: f64,
    pub filled_gross: f64,
}

#[derive(Debug)]
pub struct AccountDataBalance {
    pub asset: String, // can be smolstr e.g. btc
    pub exchange: ExchangeId,
    pub balance: Balance,
}

impl AccountDataBalance {
    pub fn new(asset: String, exchange: ExchangeId, balance: Balance) -> Self {
        Self {
            asset,
            exchange,
            balance,
        }
    }
}

impl From<&AccountDataBalance> for SpotBalanceId {
    fn from(asset_balance: &AccountDataBalance) -> Self {
        SpotBalanceId(format!("{}_{}", asset_balance.asset, asset_balance.exchange).to_lowercase())
    }
}

#[derive(Debug)]
pub struct AccountDataBalanceDelta {
    pub asset: String,
    pub exchange: ExchangeId,
    pub total: f64,
    pub available: f64, // only used for margin will be zero if spot
}

#[derive(Debug)]
pub enum AccountData {
    Order(AccountDataOrder),
    BalanceVec(Vec<AccountDataBalance>),
    Balance(AccountDataBalance),
    BalanceDelta(AccountDataBalanceDelta),
}
