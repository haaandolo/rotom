use chrono::{DateTime, Utc};
use rotom_data::shared::subscription_models::ExchangeId;
use serde::Deserialize;

use super::{
    balance::{Balance, SpotBalanceId},
    execution_request::ExecutionRequest,
    Side,
};

/*----- */
// Order Response
/*----- */
#[derive(Debug)]
pub enum ExecutionResponse {
    Order(OrderResponse),
    BalanceVec(Vec<AccountBalance>),
    Balance(AccountBalance),
    BalanceDelta(AccountBalanceDelta),
    // Send request back to oms if cannot be executed
    ExecutionError(ExecutionRequest),
}

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

/*
pub enum ExchangeOrderState {
    Open(Open),
    FullyFilled,
    Cancelled(Cancelled),
    Rejected(Option<String>),
    Expired,
}
*/

/*----- */
// Account Data - Order
/*----- */
#[derive(Debug)]
pub struct OrderResponse {
    pub exchange: ExchangeId,
    pub asset: String, // smol
    pub client_order_id: String,
    // Executed price for this order. E.g. if filled with multiple price levels, the price represents the price this certain order got executed
    pub current_executed_price: f64,
    // Excuted quantity for this order. E.g. if order is filled with mulitple limit orders, then this figure represents the quantity for this order and not the aggregate
    pub current_executed_quantity: f64,
    pub cumulative_base: f64,
    pub cumulative_quote: f64,
    pub status: OrderStatus,
    pub execution_time: DateTime<Utc>,
    pub side: Side,
    // Fee is not cumulative, and relates to this particular trade. E.g. if filled on multiple orders, this fee represents the amount paid for a single fill and not the aggregate
    pub fee: f64,
}

/*----- */
// Account Data - Balance
/*----- */
#[derive(Debug, Clone)]
pub struct AccountBalance {
    // note asset is a single currency and not a pair e.g OP not OP_USDT
    pub asset: String, // can be smolstr
    pub exchange: ExchangeId,
    pub balance: Balance,
}

impl AccountBalance {
    pub fn new(asset: String, exchange: ExchangeId, balance: Balance) -> Self {
        Self {
            asset,
            exchange,
            balance,
        }
    }
}

impl From<&AccountBalance> for SpotBalanceId {
    fn from(asset_balance: &AccountBalance) -> Self {
        SpotBalanceId(format!("{}_{}", asset_balance.exchange, asset_balance.asset,).to_lowercase())
    }
}

/*----- */
// Account Data - Balance Delta
/*----- */
#[derive(Debug, Clone)]
pub struct AccountBalanceDelta {
    // note asset is a single currency and not a pair e.g OP not OP_USDT
    pub asset: String, // smol
    pub exchange: ExchangeId,
    pub total: f64,
    pub available: f64, // only used for margin will be zero if spot
}

impl From<&AccountBalanceDelta> for SpotBalanceId {
    fn from(asset_balance: &AccountBalanceDelta) -> Self {
        SpotBalanceId(format!("{}_{}", asset_balance.exchange, asset_balance.asset,).to_lowercase())
    }
}
