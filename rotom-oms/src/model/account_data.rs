use chrono::{DateTime, Utc};
use rotom_data::{shared::subscription_models::ExchangeId, ExchangeAssetId};
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

/*----- */
// Account Data - Order
/*----- */
#[derive(Debug)]
pub struct AccountDataOrder {
    pub exchange: ExchangeId,
    pub client_order_id: String,
    pub asset: String, // smol
    // Executed price for this order. E.g. if filled with multiple price levels, the price represents the price this certain order got executed
    pub price: f64,
    // Excuted quantity for this order. E.g. if order is filled with mulitple limit orders, then this figure represents the quantity for this order and not the aggregate
    pub quantity: f64,
    pub status: OrderStatus,
    pub execution_time: DateTime<Utc>,
    pub side: Side,
    // Fee is not cumulative, and relates to this particular trade. E.g. if filled on multiple orders, this fee represents the amount paid for a single fill and not the aggregate
    pub fee: f64,
    // Aggregate quote fill value, this is a cumulative figure. E.g. if filled with muliple limit orders, this represents the total filled quote amount for the order as a whole. We wanted $50 worth and got 2 limit order fills or $10, so this figure will be $20.
    pub filled_gross: f64,
}

impl From<&AccountDataOrder> for ExchangeAssetId {
    fn from(order: &AccountDataOrder) -> Self {
        ExchangeAssetId(format!("{}_{}", order.exchange.as_str(), order.asset).to_uppercase())
    }
}

/*----- */
// Account Data - Balance
/*----- */
#[derive(Debug, Clone)]
pub struct AccountDataBalance {
    // note asset is a single currency and not a pair e.g OP not OP_USDT
    pub asset: String, // can be smolstr
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
        SpotBalanceId(format!("{}_{}", asset_balance.exchange, asset_balance.asset,).to_lowercase())
    }
}

impl From<&AccountDataBalance> for ExchangeAssetId {
    fn from(balance: &AccountDataBalance) -> Self {
        ExchangeAssetId(format!("{}_{}", balance.exchange.as_str(), balance.asset).to_uppercase())
    }
}

/*----- */
// Account Data - Balance Delta
/*----- */
#[derive(Debug, Clone)]
pub struct AccountDataBalanceDelta {
    // note asset is a single currency and not a pair e.g OP not OP_USDT
    pub asset: String, // smol
    pub exchange: ExchangeId,
    pub total: f64,
    pub available: f64, // only used for margin will be zero if spot
}

impl From<&AccountDataBalanceDelta> for SpotBalanceId {
    fn from(asset_balance: &AccountDataBalanceDelta) -> Self {
        SpotBalanceId(format!("{}_{}", asset_balance.exchange, asset_balance.asset,).to_lowercase())
    }
}

impl From<&AccountDataBalanceDelta> for ExchangeAssetId {
    fn from(balance: &AccountDataBalanceDelta) -> Self {
        ExchangeAssetId(format!("{}_{}", balance.exchange.as_str(), balance.asset).to_uppercase())
    }
}

#[derive(Debug)]
pub enum ExecutionResponse {
    Subscribed(ExchangeId), // Used to let traders know it has succesfully subscribed to ExecutionManager
    Order(AccountDataOrder),
    BalanceVec(Vec<AccountDataBalance>),
    Balance(AccountDataBalance),
    BalanceDelta(AccountDataBalanceDelta),
}
