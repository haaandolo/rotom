use hmac::Mac;
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::execution::exchange_client::binance::binance_client::BinanceSymbol;
use crate::execution::exchange_client::MethodGenerator;
use crate::portfolio::OrderType;
use crate::{
    execution::exchange_client::{HmacSha256, ParamString, SignatureGenerator},
    portfolio::OrderEvent,
};

/*----- */
// Binance order enums
/*----- */
/*
GTC: good till cancelled, an order will be on the book unless the order is cancelled
IOC: immediate or cancel, an order will try to fill the order as much as it can before the order expires
FOK: fill or kill, an order will expire if the full order cannot be filled upon execution
*/
#[derive(Debug, Deserialize, Serialize)]
pub enum BinanceTimeInForce {
    GTC,
    IOC,
    FOK,
}

impl AsRef<str> for BinanceTimeInForce {
    fn as_ref(&self) -> &str {
        match self {
            BinanceTimeInForce::GTC => "GTC",
            BinanceTimeInForce::IOC => "IOC",
            BinanceTimeInForce::FOK => "FOK",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum BinanceSide {
    BUY,
    SELL,
}

impl AsRef<str> for BinanceSide {
    fn as_ref(&self) -> &str {
        match self {
            BinanceSide::BUY => "BUY",
            BinanceSide::SELL => "SELL",
        }
    }
}

/*----- */
// Binance API authentication params
/*----- */
pub struct BinanceAuthParams;

impl BinanceAuthParams {
    pub const SECRET: &'static str = env!("BINANCE_API_SECRET");
    pub const KEY: &'static str = env!("BINANCE_API_KEY");
    pub const PRIVATE_ENDPOINT: &'static str = "wss://ws-api.binance.com:443/ws-api/v3";
}

pub fn generate_signature(request_str: String) -> String {
    let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
        .expect("Could not generate HMAC for Binance");
    mac.update(request_str.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/*----- */
// Binance order types
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub enum BinanceOrders {
    Limit(BinanceLimit),
    Market(BinanceMarket),
}

/*----- */
// Binance new order
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceNewOrder<OrderKind> {
    pub id: Uuid,
    pub method: &'static str,
    pub params: OrderKind,
}

impl<OrderKind> From<OrderEvent> for BinanceNewOrder<OrderKind> {
    fn from(order_event: OrderEvent) -> Self {
        return match &order_event.order_type {
            OrderType::Market => {
                let params = BinanceMarket::from(&order_event);
                Self {
                    id: Uuid::new_v4(),
                    method: params.get_method(),
                    params,
                }
            }
            OrderType::Limit => BinanceOrders::Limit(BinanceLimit::from(&order_event)),
        };

        // Self {
        //     id: Uuid::new_v4(),
        //     method: ,
        //     params:
        // }
    }
}

/*----- */
// Binance Mandatory fields
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceMandatory {
    pub symbol: String,
    pub side: BinanceSide,
    pub r#type: String,
    pub api_key: String,
    pub signature: Option<String>,
    pub timestamp: i64,
}

/*----- */
// Binance limit
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceLimit {
    pub symbol: String,
    pub side: BinanceSide,
    pub r#type: String,
    pub api_key: String,
    pub signature: Option<String>,
    pub timestamp: i64,
    #[serde(rename(serialize = "timeInForce"))]
    pub time_in_force: BinanceTimeInForce,
    pub price: f64,
    pub quantity: f64,
}

impl MethodGenerator for BinanceLimit {
    fn get_method(self) -> &'static str {
        "order.place"
    }
}

impl From<&OrderEvent> for BinanceLimit {
    fn from(order_event: &OrderEvent) -> Self {
        Self {
            symbol: BinanceSymbol::from(&order_event.instrument).0,
            side: BinanceSide::from(order_event.decision),
            r#type: order_event.order_type.as_ref().to_uppercase(),
            time_in_force: BinanceTimeInForce::GTC, // TODO: make dynamic
            price: order_event.market_meta.close,
            quantity: order_event.quantity,
            api_key: String::from(BinanceAuthParams::KEY),
            signature: None,
            timestamp: order_event.time.timestamp_millis(),
        }
    }
}

// impl From<&BinanceLimit> for ParamString {
//     fn from(new_order: &BinanceLimit) -> ParamString {
//         ParamString(format!(
//             "apiKey={}&price={:?}&quantity={:?}&side={}&symbol={}&timeInForce={}&timestamp={}&type={}",
//             new_order.params.api_key,
//             new_order.params.price,
//             new_order.params.quantity,
//             new_order.params.side.as_ref(),
//             new_order.params.symbol,
//             new_order.params.time_in_force.as_ref(),
//             new_order.params.timestamp,
//             new_order.params.r#type,
//         ))
//     }
// }

/*----- */
// Binance market
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceMarket {
    pub symbol: String,
    pub side: BinanceSide,
    pub r#type: String,
    pub api_key: String,
    pub signature: Option<String>,
    pub timestamp: i64,
    pub quantity: f64,
}

impl MethodGenerator for BinanceMarket {
    fn get_method(self) -> &'static str {
        "order.place"
    }
}

impl From<&OrderEvent> for BinanceMarket {
    fn from(order_event: &OrderEvent) -> Self {
        Self {
            symbol: BinanceSymbol::from(&order_event.instrument).0,
            side: BinanceSide::from(order_event.decision),
            r#type: order_event.order_type.as_ref().to_uppercase(),
            quantity: order_event.quantity,
            api_key: String::from(BinanceAuthParams::KEY),
            signature: None,
            timestamp: order_event.time.timestamp_millis(),
        }
    }
}
// impl From<&BinanceMarket> for ParamString {
//     fn from(new_order: &BinanceMarket) -> ParamString {
//         ParamString(format!(
//             "apiKey={}&quantity={:?}&side={}&symbol={}&timestamp={}&type={}",
//             new_order.params.api_key,
//             new_order.params.quantity,
//             new_order.params.side.as_ref(),
//             new_order.params.symbol,
//             new_order.params.timestamp,
//             new_order.params.r#type,
//         ))
//     }
// }

//

#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceNewOrderParams {
    pub symbol: String,
    pub side: BinanceSide,
    pub r#type: String,
    #[serde(rename(serialize = "timeInForce"))]
    pub time_in_force: BinanceTimeInForce,
    pub price: f64,
    pub quantity: f64,
    #[serde(rename(serialize = "apiKey"))]
    pub api_key: String,
    pub signature: Option<String>,
    pub timestamp: i64,
}

// impl From<&BinanceNewOrder> for ParamString {
//     fn from(new_order: &BinanceNewOrder) -> ParamString {
//         ParamString(format!(
//             "apiKey={}&price={:?}&quantity={:?}&side={}&symbol={}&timeInForce={}&timestamp={}&type={}",
//             new_order.params.api_key,
//             new_order.params.price,
//             new_order.params.quantity,
//             new_order.params.side.as_ref(),
//             new_order.params.symbol,
//             new_order.params.time_in_force.as_ref(),
//             new_order.params.timestamp,
//             new_order.params.r#type,
//         ))
//     }
// }

impl From<Decision> for BinanceSide {
    fn from(decision: Decision) -> Self {
        match decision {
            Decision::Long => BinanceSide::BUY,
            Decision::CloseLong => BinanceSide::SELL,
            Decision::Short => BinanceSide::SELL,
            Decision::CloseShort => BinanceSide::BUY,
        }
    }
}

impl From<&OrderEvent> for BinanceNewOrderParams {
    fn from(order_event: &OrderEvent) -> Self {
        Self {
            symbol: BinanceSymbol::from(&order_event.instrument).0,
            side: BinanceSide::from(order_event.decision),
            r#type: order_event.order_type.as_ref().to_uppercase(),
            time_in_force: BinanceTimeInForce::GTC, // TODO: make dynamic
            price: order_event.market_meta.close,
            quantity: order_event.quantity,
            api_key: String::from(BinanceAuthParams::KEY),
            signature: None,
            timestamp: order_event.time.timestamp_millis(),
        }
    }
}

// impl From<OrderEvent> for BinanceNewOrder {
//     fn from(order_event: OrderEvent) -> Self {
//         Self {
//             id: Uuid::new_v4(),
//             method: String::from("order.test"), // TODO: make this dynamic
//             params: BinanceNewOrderParams::from(&order_event),
//         }
//     }
// }

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
