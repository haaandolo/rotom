use chrono::Utc;
use serde::Serialize;
use serde_urlencoded;

use crate::execution::error::RequestBuildError;
use crate::execution::exchange_client::binance::auth::{generate_signature, BinanceAuthParams};
use crate::portfolio::OrderType;
use crate::{portfolio::OrderEvent};

use super::{BinanceSide, BinanceSymbol, BinanceTimeInForce};

/*----- */
// Binance New Order Params
/*----- */
// Mandatory field: symbol, side, type, apiKey, signature, timestamp
// IMPORTANT!!! Field name HAVE to be alphabetical
#[derive(Debug, Serialize)]
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

impl BinanceNewOrderParams {
    pub fn new(order_event: &OrderEvent) -> Self {
        match &order_event.order_type {
            OrderType::Limit => Self::limit_order(order_event),
            OrderType::Market => Self::market_order(order_event)
        }
    }

    fn limit_order(order_event: &OrderEvent) -> Self {
        BinanceNewOrderParamsBuilder::default()
            .price(order_event.market_meta.close)
            .quantity(order_event.quantity)
            .side(BinanceSide::from(order_event.decision))
            .symbol(BinanceSymbol::from(&order_event.instrument).0)
            .time_in_force(BinanceTimeInForce::GTC) // TODO
            .r#type(order_event.order_type.as_ref().to_uppercase())
            .sign()
            .build()
            .unwrap() // TODO
    }

    fn market_order(order_event: &OrderEvent) -> Self {
        BinanceNewOrderParamsBuilder::default()
                .quantity(order_event.quantity)
                .side(BinanceSide::from(order_event.decision))
                .symbol(BinanceSymbol::from(&order_event.instrument).0)
                .r#type(order_event.order_type.as_ref().to_uppercase())
                .sign()
                .build()
                .unwrap() // TODO
    }
}

/*----- */
// Binance New Order Param Builder
/*----- */
#[derive(Debug, Serialize)]
pub struct BinanceNewOrderParamsBuilder {
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
    pub side: Option<BinanceSide>,
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
    pub symbol: Option<String>,
    #[serde(rename(serialize = "timeInForce"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<BinanceTimeInForce>,
    pub timestamp: i64,
    pub r#type: Option<String>,
    #[serde(rename(serialize = "trailingDelta"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_delta: Option<u64>,
}

impl Default for BinanceNewOrderParamsBuilder {
    fn default() -> Self {
        Self {
            api_key: BinanceAuthParams::KEY, // mandatory
            iceberg_qty: None,
            new_client_order_id: None,
            new_order_resp_type: None, // TODO: change to enum
            price: None,
            quantity: None,
            quote_order_qty: None,
            recv_window: None, // value cannot be > 60_000
            self_trade_prevention_mode: None, // TODO: change to enum
            side: None, // mandatory
            signature: None,
            stop_price: None,
            strategy_id: None,
            strategy_type: None,
            symbol: None, // mandatory
            time_in_force: None,
            timestamp: Utc::now().timestamp_millis(), // mandatory
            r#type: None, // Mandatory
            trailing_delta: None,
        }
    }
}

impl BinanceNewOrderParamsBuilder {
    pub fn iceberg_qty(self, iceberg_qty: u64) -> Self {
        Self {
            iceberg_qty: Some(iceberg_qty),
            ..self
        }
    }

    pub fn new_client_order_id(self, new_client_order_id: String) -> Self {
        Self {
            new_client_order_id: Some(new_client_order_id),
            ..self
        }
    }

    pub fn new_order_resp_type(self, new_order_resp_type: u32) -> Self {
        Self {
            new_order_resp_type: Some(new_order_resp_type),
            ..self
        }
    }

    pub fn price(self, price: f64) -> Self {
        Self {
            price: Some(price),
            ..self
        }
    }

    pub fn quantity(self, quantity: f64) -> Self {
        Self {
            quantity: Some(quantity),
            ..self
        }
    }

    pub fn quote_order_qty(self, quote_order_qty: f64) -> Self {
        Self {
            quote_order_qty: Some(quote_order_qty),
            ..self
        }
    }

    pub fn recv_window(self, recv_window: u64) -> Self {
        Self {
            recv_window: Some(recv_window),
            ..self
        }
    }

    pub fn self_trade_prevention_mode(self, self_trade_prevention_mode: u64) -> Self {
        Self {
            recv_window: Some(self_trade_prevention_mode),
            ..self
        }
    }

    pub fn side(self, side: BinanceSide) -> Self {
        Self {
            side: Some(side),
            ..self
        }
    }

    pub fn sign(self) -> Self {
        let signature = generate_signature(serde_urlencoded::to_string(&self).unwrap()); // TODO
        Self {
            signature: Some(signature),
            ..self
        }
    }

    pub fn stop_price(self, stop_price: f64) -> Self {
        Self {
            stop_price: Some(stop_price),
            ..self
        }
    }

    pub fn strategy_id(self, strategy_id: u64) -> Self {
        Self {
            strategy_id: Some(strategy_id),
            ..self
        }
    }

    pub fn strategy_type(self, strategy_type: u64) -> Self {
        Self {
            strategy_type: Some(strategy_type),
            ..self
        }
    }

    pub fn symbol(self, symbol: String) -> Self {
        Self {
            symbol: Some(symbol),
            ..self
        }
    }

    pub fn time_in_force(self, time_in_force: BinanceTimeInForce) -> Self {
        Self {
            time_in_force: Some(time_in_force),
            ..self
        }
    }

    pub fn r#type(self, r#type: String) -> Self {
        Self {
            r#type: Some(r#type),
            ..self
        }
    }

    pub fn trailing_delta(self, trailing_delta: u64) -> Self {
        Self {
            trailing_delta: Some(trailing_delta),
            ..self
        }
    }

    pub fn build(self) -> Result<BinanceNewOrderParams, RequestBuildError> {
        Ok(BinanceNewOrderParams {
            api_key: self.api_key,
            iceberg_qty: self.iceberg_qty,
            new_client_order_id: self.new_client_order_id,
            new_order_resp_type: self.new_order_resp_type,
            price: self.price,
            quantity: self.quantity,
            quote_order_qty: self.quote_order_qty,
            recv_window: self.recv_window,
            self_trade_prevention_mode: self.self_trade_prevention_mode,
            side: self.side.ok_or(RequestBuildError::MandatoryField {
                exchange: "Binance",
                request: "new order",
                field: "side",
            })?,
            signature: self.signature,
            stop_price: self.stop_price,
            strategy_id: self.strategy_id,
            strategy_type: self.strategy_type,
            symbol: self.symbol.ok_or(RequestBuildError::MandatoryField {
                exchange: "Binance",
                request: "new order",
                field: "symbol",
            })?,
            time_in_force: self.time_in_force,
            timestamp: self.timestamp,
            r#type: self.r#type.ok_or(RequestBuildError::MandatoryField {
                exchange: "Binance",
                request: "new order",
                field: "type",
            })?,
            trailing_delta: self.trailing_delta,
        })
    }
}

// /*----- */
// // Binance cancel and replace
// /*----- */
// #[derive(Debug, Serialize)]
// pub struct BinanceCancelReplace {
//     id: String,
//     method: String,
//     params: BinanceCancelReplaceParams,
// }

// #[derive(Debug, Serialize)]
// pub struct BinanceCancelReplaceParams {
//     symbol: String,
//     #[serde(rename(serialize = "cancelReplaceMode"))]
//     cancel_replace_mode: String,
//     #[serde(rename(serialize = "cancelOrigClientOrderId"))]
//     cancel_orig_client_order_id: String,
//     side: String,
//     r#type: String,
//     #[serde(rename(serialize = "timeInForce"))]
//     time_in_force: String,
//     price: f64,
//     quantity: f64,
//     #[serde(rename(serialize = "apiKey"))]
//     api_key: String,
//     signature: String,
//     timestamp: u64,
// }

// /*----- */
// // Binance current open orders
// /*----- */
// #[derive(Debug, Serialize)]
// pub struct BinanceCancelAll {
//     id: String,
//     method: String,
//     params: BinanceCancelAllParams,
// }

// #[derive(Debug, Serialize)]
// pub struct BinanceCancelAllParams {
//     symbol: String,
//     #[serde(rename(serialize = "apiKey"))]
//     api_key: String,
//     signature: String,
//     timestamp: u64,
// }

// /*----- */
// // Binance cancel open orders
// /*----- */
// /**/
