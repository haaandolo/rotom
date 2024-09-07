use serde::{Deserialize, Serialize};
use serde_urlencoded;
use uuid::Uuid;

use crate::execution::exchange_client::binance::auth::BinanceAuthParams;
use crate::execution::exchange_client::OrderEventConverter;
use crate::portfolio::OrderType;
use crate::{execution::exchange_client::ParamString, portfolio::OrderEvent};

use super::{BinanceSide, BinanceSymbol, BinanceTimeInForce};

/*----- */
// Binance New Order
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceNewOrder {
    id: Uuid,
    method: &'static str,
    pub params: BinanceNewOrderParams,
}

// Mandatory field: symbol, side, type, apiKey, signature, timestamp
// IMPORTANT!!! Field name HAVE to be alphabetical
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceNewOrderParams {
    #[serde(rename(serialize = "apiKey"))]
    pub api_key: &'static str,
    #[serde(rename(serialize = "icebergQty"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iceberg_qty: Option<u64>,
    #[serde(rename(serialize = "newClientOrderId"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    #[serde(rename(serialize = "newOrderRespType"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<u32>, // TODO: change to enum
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    pub quantity: Option<f64>,
    #[serde(rename(serialize = "quoteOrderQty"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_order_qty: Option<f64>,
    #[serde(rename(serialize = "recvWindow"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>, // value cannot be > 60_000
    #[serde(rename(serialize = "selfTradePreventionMode"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_mode: Option<u64>, // TODO: change to enum
    pub side: BinanceSide,
    pub signature: Option<String>,
    #[serde(rename(serialize = "stopPrice"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(rename(serialize = "strategyId"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_id: Option<u64>,
    #[serde(rename(serialize = "strategyType"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_type: Option<u64>,
    pub symbol: String,
    #[serde(rename(serialize = "timeInForce"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<BinanceTimeInForce>,
    pub timestamp: i64,
    pub r#type: String,
    #[serde(rename(serialize = "trailingDelta"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_delta: Option<u64>,
}

impl Default for BinanceNewOrderParams {
    fn default() -> Self {
        // IMPORTANT!!! Field name HAVE to be alphabetical
        Self {
            api_key: BinanceAuthParams::KEY,
            iceberg_qty: None,
            new_client_order_id: None,
            new_order_resp_type: None, // TODO: change to enum
            price: None,
            quantity: None,
            quote_order_qty: None,
            recv_window: None,                // value cannot be > 60_000
            self_trade_prevention_mode: None, // TODO: change to enum
            side: BinanceSide::BUY,
            signature: None,
            stop_price: None,
            strategy_id: None,
            strategy_type: None,
            symbol: String::default(),
            time_in_force: None,
            timestamp: i64::default(),
            r#type: String::default(),
            trailing_delta: None,
        }
    }
}

impl OrderEventConverter for BinanceNewOrder {
    type OrderKind = BinanceNewOrderParams;
    type ExchangeAuthParams = BinanceAuthParams;

    fn get_request_method() -> &'static str {
        "order.test" // TODO: change back to "order.place"
    }

    fn convert_order_event(order_event: &OrderEvent) -> BinanceNewOrderParams {
        return match order_event.order_type {
            // IMPORTANT!!! Field name HAVE to be alphabetical
            OrderType::Limit => BinanceNewOrderParams {
                api_key: BinanceAuthParams::KEY,
                price: Some(order_event.market_meta.close),
                quantity: Some(order_event.quantity),
                side: BinanceSide::from(order_event.decision),
                signature: None,
                symbol: BinanceSymbol::from(&order_event.instrument).0,
                time_in_force: Some(BinanceTimeInForce::GTC), // TODO: make dynamic
                timestamp: order_event.time.timestamp_millis(),
                r#type: order_event.order_type.as_ref().to_uppercase(),
                ..Default::default()
            },
            // IMPORTANT!!! Field name HAVE to be alphabetical
            OrderType::Market => BinanceNewOrderParams {
                api_key: BinanceAuthParams::KEY,
                quantity: Some(order_event.quantity),
                side: BinanceSide::from(order_event.decision),
                signature: None,
                symbol: BinanceSymbol::from(&order_event.instrument).0,
                timestamp: order_event.time.timestamp_millis(),
                r#type: order_event.order_type.as_ref().to_uppercase(),
                ..Default::default()
            },
        };
    }

    fn get_query_param(&self) -> ParamString {
        ParamString(serde_urlencoded::to_string(&self.params).unwrap_or_default())
    }
}

impl BinanceNewOrder {
    pub fn new(order_event: &OrderEvent) -> BinanceNewOrder {
        Self {
            id: Uuid::new_v4(),
            method: Self::get_request_method(),
            params: Self::convert_order_event(order_event),
        }
    }
}

/*----- */
// Binance cancel order
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceCancelOrder {
    id: String,
    method: String,
    params: BinanceCancelOrderParams,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceCancelOrderParams {
    symbol: String,
    #[serde(rename(serialize = "origClientOrderId"))]
    orig_client_order_id: String,
    #[serde(rename(serialize = "apiKey"))]
    api_key: String,
    signature: String,
    timestamp: u64,
}

/*----- */
// Binance cancel and replace
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceCancelReplace {
    id: String,
    method: String,
    params: BinanceCancelReplaceParams,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceCancelReplaceParams {
    symbol: String,
    #[serde(rename(serialize = "cancelReplaceMode"))]
    cancel_replace_mode: String,
    #[serde(rename(serialize = "cancelOrigClientOrderId"))]
    cancel_orig_client_order_id: String,
    side: String,
    r#type: String,
    #[serde(rename(serialize = "timeInForce"))]
    time_in_force: String,
    price: f64,
    quantity: f64,
    #[serde(rename(serialize = "apiKey"))]
    api_key: String,
    signature: String,
    timestamp: u64,
}

/*----- */
// Binance current open orders
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceCancelAll {
    id: String,
    method: String,
    params: BinanceCancelAllParams,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceCancelAllParams {
    symbol: String,
    #[serde(rename(serialize = "apiKey"))]
    api_key: String,
    signature: String,
    timestamp: u64,
}

/*----- */
// Binance cancel open orders
/*----- */
/**/

/*----- */
// Examples
/*----- */
/*
Example: new order
{
    "id": "56374a46-3061-486b-a311-99ee972eb648",
    "method": "order.place",
    "params": {
      "symbol": "BTCUSDT",
      "side": "SELL",
      "type": "LIMIT",
      "timeInForce": "GTC",
      "price": "23416.10000000",
      "quantity": "0.00847000",
      "apiKey": "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A",
      "signature": "15af09e41c36f3cc61378c2fbe2c33719a03dd5eba8d0f9206fbda44de717c88",
      "timestamp": 1660801715431
    }
}

Example: cancel order
{
    "id": "5633b6a2-90a9-4192-83e7-925c90b6a2fd",
    "method": "order.cancel",
    "params": {
      "symbol": "BTCUSDT",
      "origClientOrderId": "4d96324ff9d44481926157",
      "apiKey": "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A",
      "signature": "33d5b721f278ae17a52f004a82a6f68a70c68e7dd6776ed0be77a455ab855282",
      "timestamp": 1660801715830
    }
}

Example: cancel or replace
{
    "id": "99de1036-b5e2-4e0f-9b5c-13d751c93a1a",
    "method": "order.cancelReplace",
    "params": {
      "symbol": "BTCUSDT",
      "cancelReplaceMode": "ALLOW_FAILURE",
      "cancelOrigClientOrderId": "4d96324ff9d44481926157",
      "side": "SELL",
      "type": "LIMIT",
      "timeInForce": "GTC",
      "price": "23416.10000000",
      "quantity": "0.00847000",
      "apiKey": "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A",
      "signature": "7028fdc187868754d25e42c37ccfa5ba2bab1d180ad55d4c3a7e2de643943dc5",
      "timestamp": 1660813156900
    }
}

Example: cancel all orders for a given symbol
{
    "id": "778f938f-9041-4b88-9914-efbf64eeacc8",
    "method": "openOrders.cancelAll"
    "params": {
      "symbol": "BTCUSDT",
      "apiKey": "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A",
      "signature": "773f01b6e3c2c9e0c1d217bc043ce383c1ddd6f0e25f8d6070f2b66a6ceaf3a5",
      "timestamp": 1660805557200
    }
}
*/
