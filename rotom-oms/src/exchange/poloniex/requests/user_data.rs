use rotom_data::shared::de::de_str;
use serde::{Deserialize, Serialize};

use super::{PoloniexOrderStatus, PoloniexOrderType, PoloniexSide};

/*----- */
// Poloniex User Data - Orders
/*----- */

#[derive(Debug, Deserialize)]
pub struct PoloniexUserDataOrder {
    pub channel: String,
    pub data: [PoloniexUserDataOrderParams; 1],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoloniexUserDataOrderParams {
    pub symbol: String,
    pub r#type:  PoloniexOrderType,
    #[serde(deserialize_with = "de_str")]
    pub quantity: f64,
    #[serde(deserialize_with = "de_str")]
    pub order_id: u64,
    #[serde(deserialize_with = "de_str")]
    pub trade_fee: f64,
    pub client_order_id: String,
    pub account_type: String,
    pub fee_currency: String,
    pub event_type: PoloniexOrderEventType,
    pub source: String,
    pub side: PoloniexSide,
    #[serde(deserialize_with = "de_str")]
    pub filled_quantity: f64,
    #[serde(deserialize_with = "de_str")]
    pub filled_amount: f64,
    pub match_role: String,
    pub state: PoloniexOrderStatus,
    pub trade_time: u64,
    #[serde(deserialize_with = "de_str")]
    pub trade_amount: f64,
    #[serde(deserialize_with = "de_str")]
    pub order_amount: f64,
    pub create_time: u64,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub trade_qty: f64,
    #[serde(deserialize_with = "de_str")]
    pub trade_price: f64,
    #[serde(deserialize_with = "de_str")]
    pub trade_id: u64,
    pub ts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PoloniexOrderEventType {
    Place,
    Trade,
    Canceled,
}

/*----- */
// Poloniex User Data - Balance
/*----- */
#[derive(Debug, Deserialize)]
pub struct PoloniexUserDataBalance {
    pub channel: String,
    pub data: [PoloniexUserDataBalanceParams; 1],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoloniexUserDataBalanceParams {
    pub change_time: u64,
    #[serde(deserialize_with = "de_str")]
    pub account_id: u64,
    #[serde(deserialize_with = "de_str")]
    pub account_type: String,
    pub event_type: PoloniexBalanceEventType,
    #[serde(deserialize_with = "de_str")]
    pub available: f64,
    pub currency: String,
    pub id: u64,
    pub user_id: u64,
    #[serde(deserialize_with = "de_str")]
    pub hold: f64,
    pub ts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum PoloniexBalanceEventType {
    PlaceOrder,
    CanceledOrder,
    MatchOrder,
    TransferIn,
    TransferOut,
    Deposit,
    Withdraw,
}

/*----- */
// Poloniex User Data Response
/*----- */
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PoloniexUserData {
    Order(PoloniexUserDataOrder),
    Balance(PoloniexUserDataBalance),
}