use chrono::{DateTime, Utc};
use rotom_data::shared::de::de_str;
use rotom_data::shared::de::de_u64_epoch_ms_as_datetime_utc;
use rotom_data::shared::subscription_models::ExchangeId;
use serde::{Deserialize, Serialize};

use crate::model::account_data::AccountDataBalance;
use crate::model::account_data::AccountDataOrder;
use crate::model::account_data::ExecutionResponse;
use crate::model::account_data::OrderStatus;
use crate::model::balance::Balance;
use crate::model::OrderKind;
use crate::model::Side;

/*----- */
// Poloniex User Data - Orders
/*----- */
#[derive(Debug, Deserialize)]
pub struct PoloniexAccountDataOrder {
    pub channel: String,
    pub data: [PoloniexAccountDataOrderParams; 1],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoloniexAccountDataOrderParams {
    pub symbol: String,    // symbol name
    pub r#type: OrderKind, // market, limit, limit maker
    #[serde(deserialize_with = "de_str")]
    pub quantity: f64, // number of base units for this order
    pub order_id: String,  // order id
    #[serde(deserialize_with = "de_str")]
    pub trade_fee: f64, // fee amount for the trade
    pub client_order_id: String, // user specfied id
    pub account_type: String, // SPOT
    pub fee_currency: String, // fee currency name
    pub event_type: PoloniexOrderEventType, // place, trade, canceled
    pub source: String,    // web, app, api
    pub side: Side,        // BUY, SELL
    #[serde(deserialize_with = "de_str")]
    pub filled_quantity: f64, // base unit filled in this order
    #[serde(deserialize_with = "de_str")]
    pub filled_amount: f64, // quote unit filled in this order
    pub match_role: String, // maker, taker
    pub state: OrderStatus, // New, partially_filled, filled, pending_cancel, partially_canceled, canceled, failed
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub trade_time: DateTime<Utc>, // time the trade was executed
    #[serde(deserialize_with = "de_str")]
    pub trade_amount: f64, // number  of quote units for a trade
    #[serde(deserialize_with = "de_str")]
    pub order_amount: f64, // number of quote units for this order
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub create_time: DateTime<Utc>, // time the record was created
    #[serde(deserialize_with = "de_str")]
    pub price: f64, // set price of the order
    #[serde(deserialize_with = "de_str")]
    pub trade_qty: f64, // number of base units for a trade
    #[serde(deserialize_with = "de_str")]
    pub trade_price: f64, // price of the trade
    #[serde(deserialize_with = "de_str")]
    pub trade_id: u64, // id of the trade
    pub ts: u64,            // time the record was pushed
}

impl From<PoloniexAccountDataOrder> for AccountDataOrder {
    fn from(mut order: PoloniexAccountDataOrder) -> Self {
        Self {
            exchange: ExchangeId::PoloniexSpot,
            client_order_id: std::mem::take(&mut order.data[0].order_id),
            asset: std::mem::take(&mut order.data[0].symbol),
            current_executed_price: order.data[0].trade_price,
            current_executed_quantity: order.data[0].trade_qty,
            cumulative_base: order.data[0].filled_quantity,
            cumulative_quote: order.data[0].filled_amount,
            status: order.data[0].state,
            execution_time: order.data[0].create_time,
            side: order.data[0].side,
            fee: order.data[0].trade_fee,
        }
    }
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
pub struct PoloniexAccountDataBalance {
    pub channel: String,
    pub data: [PoloniexAccountDataBalanceParams; 1],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoloniexAccountDataBalanceParams {
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

impl From<PoloniexAccountDataBalance> for AccountDataBalance {
    fn from(mut account_balance: PoloniexAccountDataBalance) -> AccountDataBalance {
        AccountDataBalance {
            asset: std::mem::take(&mut account_balance.data[0].currency), // when changed to small string, can rm std::mem::take
            exchange: ExchangeId::PoloniexSpot,
            balance: Balance {
                total: account_balance.data[0].available,
                available: 0.0,
            },
        }
    }
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
pub enum PoloniexAccountEvents {
    Order(PoloniexAccountDataOrder),
    Balance(PoloniexAccountDataBalance),
}

impl From<PoloniexAccountEvents> for ExecutionResponse {
    fn from(account_events: PoloniexAccountEvents) -> Self {
        match account_events {
            PoloniexAccountEvents::Order(order) => {
                ExecutionResponse::Order(AccountDataOrder::from(order))
            }
            PoloniexAccountEvents::Balance(balance) => {
                ExecutionResponse::Balance(AccountDataBalance::from(balance))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_poloniex_user_data_response() {
        let response = "{\"channel\":\"balances\",\"data\":[{\"id\":391789032125874177,\"accountId\":\"292264758130978818\",\"available\":\"1.34476247912\",\"accountType\":\"SPOT\",\"hold\":\"7.5\",\"changeTime\":1734376184347,\"userId\":1887604,\"currency\":\"USDT\",\"eventType\":\"PLACE_ORDER\",\"ts\":1734376184352}]}";
        let response_de = serde_json::from_str::<PoloniexAccountEvents>(response);
        let mut _result = false;

        match response_de {
            Ok(_) => _result = true,
            Err(_) => _result = false,
        }

        assert!(_result)
    }
}

/*----- */
// Examples
/*----- */
/*
### Note ###
Asset & price field of AccountDataOrder is wrong, example copied before field was updated

### Limit buy ###
"{\"channel\":\"orders\",\"data\":[{\"orderId\":\"393344737152782336\",\"tradeId\":\"0\",\"clientOrderId\":\"\",\"accountType\":\"SPOT\",\"eventType\":\"place\",\"symbol\":\"OP_USDT\",\"side\":\"BUY\",\"type\":\"LIMIT\",\"price\":\"1.9\",\"quantity\":\"5\",\"state\":\"NEW\",\"createTime\":1734747093330,\"tradeTime\":0,\"tradePrice\":\"0\",\"tradeQty\":\"0\",\"feeCurrency\":\"\",\"tradeFee\":\"0\",\"tradeAmount\":\"0\",\"filledQuantity\":\"0\",\"filledAmount\":\"0\",\"ts\":1734747093363,\"source\":\"WEB\",\"orderAmount\":\"0\",\"matchRole\":\"\"}]}",
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393344737152782336",
    asset: "OP_USDT",
    price: 0.0,
    quantity: 5.0,
    status: New,
    execution_time: 2024-12-21T02:11:33.330Z,
    side: Buy,
    fee: 0.0,
    filled_gross: 0.0,
}

"{\"channel\":\"orders\",\"data\":[{\"orderId\":\"393344737152782336\",\"tradeId\":\"106864093\",\"clientOrderId\":\"\",\"accountType\":\"SPOT\",\"eventType\":\"trade\",\"symbol\":\"OP_USDT\",\"side\":\"BUY\",\"type\":\"LIMIT\",\"price\":\"1.9\",\"quantity\":\"5\",\"state\":\"FILLED\",\"createTime\":1734747093330,\"tradeTime\":1734747093356,\"tradePrice\":\"1.8995\",\"tradeQty\":\"5\",\"feeCurrency\":\"OP\",\"tradeFee\":\"0.01\",\"tradeAmount\":\"9.4975\",\"filledQuantity\":\"5\",\"filledAmount\":\"9.4975\",\"ts\":1734747093403,\"source\":\"WEB\",\"orderAmount\":\"0\",\"matchRole\":\"TAKER\"}]}",
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393344737152782336",
    asset: "OP_USDT",
    price: 9.4975,
    quantity: 5.0,
    status: Filled,
    execution_time: 2024-12-21T02:11:33.330Z,
    side: Buy,
    fee: 0.01,
    filled_gross: 9.4975,
}

### Limit sell ###
"{\"channel\":\"orders\",\"data\":[{\"orderId\":\"393345318000967680\",\"tradeId\":\"0\",\"clientOrderId\":\"\",\"accountType\":\"SPOT\",\"eventType\":\"place\",\"symbol\":\"OP_USDT\",\"side\":\"SELL\",\"type\":\"LIMIT\",\"price\":\"1.888\",\"quantity\":\"4.99\",\"state\":\"NEW\",\"createTime\":1734747231814,\"tradeTime\":0,\"tradePrice\":\"0\",\"tradeQty\":\"0\",\"feeCurrency\":\"\",\"tradeFee\":\"0\",\"tradeAmount\":\"0\",\"filledQuantity\":\"0\",\"filledAmount\":\"0\",\"ts\":1734747231846,\"source\":\"WEB\",\"orderAmount\":\"0\",\"matchRole\":\"\"}]}",
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393345318000967680",
    asset: "OP_USDT",
    price: 0.0,
    quantity: 4.99,
    status: New,
    execution_time: 2024-12-21T02:13:51.814Z,
    side: Sell,
    fee: 0.0,
    filled_gross: 0.0,
}

"{\"channel\":\"orders\",\"data\":[{\"orderId\":\"393345318000967680\",\"tradeId\":\"106865010\",\"clientOrderId\":\"\",\"accountType\":\"SPOT\",\"eventType\":\"trade\",\"symbol\":\"OP_USDT\",\"side\":\"SELL\",\"type\":\"LIMIT\",\"price\":\"1.888\",\"quantity\":\"4.99\",\"state\":\"PARTIALLY_FILLED\",\"createTime\":1734747231814,\"tradeTime\":1734747232148,\"tradePrice\":\"1.888\",\"tradeQty\":\"3.3059\",\"feeCurrency\":\"USDT\",\"tradeFee\":\"0.0124830784\",\"tradeAmount\":\"6.2415392\",\"filledQuantity\":\"3.3059\",\"filledAmount\":\"6.2415392\",\"ts\":1734747232192,\"source\":\"WEB\",\"orderAmount\":\"0\",\"matchRole\":\"MAKER\"}]}",
"{\"channel\":\"orders\",\"data\":[{\"orderId\":\"393345318000967680\",\"tradeId\":\"106865011\",\"clientOrderId\":\"\",\"accountType\":\"SPOT\",\"eventType\":\"trade\",\"symbol\":\"OP_USDT\",\"side\":\"SELL\",\"type\":\"LIMIT\",\"price\":\"1.888\",\"quantity\":\"4.99\",\"state\":\"FILLED\",\"createTime\":1734747231814,\"tradeTime\":1734747232150,\"tradePrice\":\"1.888\",\"tradeQty\":\"1.6841\",\"feeCurrency\":\"USDT\",\"tradeFee\":\"0.0063591616\",\"tradeAmount\":\"3.1795808\",\"filledQuantity\":\"4.99\",\"filledAmount\":\"9.42112\",\"ts\":1734747232202,\"source\":\"WEB\",\"orderAmount\":\"0\",\"matchRole\":\"MAKER\"}]}",
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393345318000967680",
    asset: "OP_USDT",
    price: 6.2415392, // 1.888
    quantity: 4.99, // 3.3059
    status: PartiallyFilled,
    execution_time: 2024-12-21T02:13:51.814Z,
    side: Sell,
    fee: 0.0124830784,
    filled_gross: 6.2415392,
}

AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393345318000967680",
    asset: "OP_USDT",
    price: 9.42112, // 1.888
    quantity: 4.99, // 1.6841
    status: Filled,
    execution_time: 2024-12-21T02:13:51.814Z,
    side: Sell,
    fee: 0.0063591616,
    filled_gross: 9.42112,
}

### Market buy ###
"{\"channel\":\"orders\",\"data\":[{\"orderId\":\"393345433256243200\",\"tradeId\":\"0\",\"clientOrderId\":\"\",\"accountType\":\"SPOT\",\"eventType\":\"place\",\"symbol\":\"OP_USDT\",\"side\":\"BUY\",\"type\":\"MARKET\",\"price\":\"0\",\"quantity\":\"0\",\"state\":\"NEW\",\"createTime\":1734747259293,\"tradeTime\":0,\"tradePrice\":\"0\",\"tradeQty\":\"0\",\"feeCurrency\":\"\",\"tradeFee\":\"0\",\"tradeAmount\":\"0\",\"filledQuantity\":\"0\",\"filledAmount\":\"0\",\"ts\":1734747259322,\"source\":\"WEB\",\"orderAmount\":\"3\",\"matchRole\":\"\"}]}",
"{\"channel\":\"orders\",\"data\":[{\"orderId\":\"393345433256243200\",\"tradeId\":\"106865178\",\"clientOrderId\":\"\",\"accountType\":\"SPOT\",\"eventType\":\"trade\",\"symbol\":\"OP_USDT\",\"side\":\"BUY\",\"type\":\"MARKET\",\"price\":\"0\",\"quantity\":\"0\",\"state\":\"FILLED\",\"createTime\":1734747259293,\"tradeTime\":1734747259317,\"tradePrice\":\"1.8925\",\"tradeQty\":\"1.5852\",\"feeCurrency\":\"OP\",\"tradeFee\":\"0.0031704\",\"tradeAmount\":\"2.999991\",\"filledQuantity\":\"1.5852\",\"filledAmount\":\"2.999991\",\"ts\":1734747259358,\"source\":\"WEB\",\"orderAmount\":\"3\",\"matchRole\":\"TAKER\"}]}",
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393345433256243200",
    asset: "OP_USDT",
    price: 0.0,
    quantity: 0.0,
    status: New,
    execution_time: 2024-12-21T02:14:19.293Z,
    side: Buy,
    fee: 0.0,
    filled_gross: 0.0,
}
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393345433256243200",
    asset: "OP_USDT",
    price: 2.999991,
    quantity: 0.0,
    status: Filled,
    execution_time: 2024-12-21T02:14:19.293Z,
    side: Buy,
    fee: 0.0031704,
    filled_gross: 2.999991,
}

### Market sell ###
"{\"channel\":\"orders\",\"data\":[{\"orderId\":\"393345503405989889\",\"tradeId\":\"0\",\"clientOrderId\":\"\",\"accountType\":\"SPOT\",\"eventType\":\"place\",\"symbol\":\"OP_USDT\",\"side\":\"SELL\",\"type\":\"MARKET\",\"price\":\"0\",\"quantity\":\"1.582\",\"state\":\"NEW\",\"createTime\":1734747276018,\"tradeTime\":0,\"tradePrice\":\"0\",\"tradeQty\":\"0\",\"feeCurrency\":\"\",\"tradeFee\":\"0\",\"tradeAmount\":\"0\",\"filledQuantity\":\"0\",\"filledAmount\":\"0\",\"ts\":1734747276049,\"source\":\"WEB\",\"orderAmount\":\"0\",\"matchRole\":\"\"}]}",
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393345503405989889",
    asset: "OP_USDT",
    price: 0.0,
    quantity: 1.582,
    status: New,
    execution_time: 2024-12-21T02:14:36.018Z,
    side: Sell,
    fee: 0.0,
    filled_gross: 0.0,
}

"{\"channel\":\"orders\",\"data\":[{\"orderId\":\"393345503405989889\",\"tradeId\":\"106865270\",\"clientOrderId\":\"\",\"accountType\":\"SPOT\",\"eventType\":\"trade\",\"symbol\":\"OP_USDT\",\"side\":\"SELL\",\"type\":\"MARKET\",\"price\":\"0\",\"quantity\":\"1.582\",\"state\":\"FILLED\",\"createTime\":1734747276018,\"tradeTime\":1734747276039,\"tradePrice\":\"1.8864\",\"tradeQty\":\"1.582\",\"feeCurrency\":\"USDT\",\"tradeFee\":\"0.0059685696\",\"tradeAmount\":\"2.9842848\",\"filledQuantity\":\"1.582\",\"filledAmount\":\"2.9842848\",\"ts\":1734747276079,\"source\":\"WEB\",\"orderAmount\":\"0\",\"matchRole\":\"TAKER\"}]}",
AccountDataOrder: AccountDataOrder {
    exchange: PoloniexSpot,
    client_order_id: "393345503405989889",
    asset: "OP_USDT",
    price: 2.9842848,
    quantity: 1.582,
    status: Filled,
    execution_time: 2024-12-21T02:14:36.018Z,
    side: Sell,
    fee: 0.0059685696,
    filled_gross: 2.9842848,
}
*/
