use chrono::{DateTime, Utc};
use rotom_data::shared::de::de_str;
use rotom_data::shared::de::de_u64_epoch_ms_as_datetime_utc;
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
    pub r#type: PoloniexOrderType,
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
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub trade_time: DateTime<Utc>,
    #[serde(deserialize_with = "de_str")]
    pub trade_amount: f64,
    #[serde(deserialize_with = "de_str")]
    pub order_amount: f64,
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub create_time: DateTime<Utc>,
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

// impl From<PoloniexUserDataOrder> for FillEvent {
//     fn from(order: PoloniexUserDataOrder) -> Self {
//         let order_data = &order.data[0];
//         Self {
//             time: Utc::now(), // todo
//             exchange: ExchangeId::PoloniexSpot,
//             decision: Decision::Long, // todo
//             quantity: order_data.quantity,
//             fill_value_gross: order_data.filled_amount,
//             fees: Fees {
//                 exchange: order_data.trade_fee,
//                 slippage: 0.0,
//                 network: 0.0,
//             },
//             market_meta: MarketMeta {
//                 time: Utc::now(), // todo
//                 close: order_data.price,
//             },
//             instrument: Instrument {
//                 // todo
//                 base: "op".to_string(),
//                 quote: "usdt".to_string(),
//             },
//         }
//     }
// }

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

/*

"{
    \"channel\":\"balances\",
    \"data\":[{
        \"id\":381810886546833408,
        \"accountId\":\"292264758130978818\",
        \"available\":\"5.97234541632\",
        \"accountType\":\"SPOT\",
        \"hold\":\"3\",\"changeTime\":1731997209056,
        \"userId\":1887604,
        \"currency\":\"USDT\",
        \"eventType\":\"PLACE_ORDER\",
        \"ts\":1731997209063
        }]
}",

"{
    \"channel\":\"orders\",
    \"data\":[{
        \"orderId\":\"381810886496567298\",
        \"tradeId\":\"0\",
        \"clientOrderId\":\"\",
        \"accountType\":\"SPOT\",
        \"eventType\":\"place\",
        \"symbol\":\"OP_USDT\",
        \"side\":\"BUY\",
        \"type\":\"LIMIT\",
        \"price\":\"1\",
        \"quantity\":\"3\",
        \"state\":\"NEW\",
        \"createTime\":1731997209044,
        \"tradeTime\":0,
        \"tradePrice\":\"0\",
        \"tradeQty\":\"0\",
        \"feeCurrency\":\"\",
        \"tradeFee\":\"0\",
        \"tradeAmount\":\"0\",
        \"filledQuantity\":\"0\",
        \"filledAmount\":\"0\",
        \"ts\":1731997209076,
        \"source\":\"WEB\",
        \"orderAmount\":\"0\",
        \"matchRole\":\"\"
        }]
    }",

"{
    \"channel\":\"balances\",
    \"data\":[{
        \"id\":381815419855466496,
        \"accountId\":\"292264758130978818\",
        \"available\":\"8.97234541632\",
        \"accountType\":\"SPOT\",
        \"hold\":\"0\",
        \"changeTime\":1731998289881,
        \"userId\":1887604,
        \"currency\":\"USDT\",
        \"eventType\":\"CANCELED_ORDER\",
        \"ts\":1731998289885
        }]
}",


"{
    \"channel\":\"orders\",
    \"data\":[{
        \"orderId\":\"381810886496567298\",
        \"tradeId\":\"0\",
        \"clientOrderId\":\"\",
        \"accountType\":\"SPOT\",
        \"eventType\":\"canceled\",
        \"symbol\":\"OP_USDT\",
        \"side\":\"BUY\",
        \"type\":\"LIMIT\",
        \"price\":\"1\",
        \"quantity\":\"3\",
        \"state\":\"CANCELED\",
        \"createTime\":1731997209044,
        \"tradeTime\":0,
        \"tradePrice\":\"0\",
        \"tradeQty\":\"0\",
        \"feeCurrency\":\"\",
        \"tradeFee\":\"0\",
        \"tradeAmount\":\"0\",
        \"filledQuantity\":\"0\",
        \"filledAmount\":\"0\",
        \"ts\":1731998289897,
        \"source\":\"WEB\",
        \"orderAmount\":\"0\",
        \"matchRole\":\"\"
        }]
}",
*/
