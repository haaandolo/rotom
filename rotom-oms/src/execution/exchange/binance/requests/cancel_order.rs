use chrono::Utc;
use rotom_data::protocols::http::rest_request::RestRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

use super::{BinanceOrderStatus, BinanceSide, BinanceTimeInForce};
use crate::execution::{error::RequestBuildError, exchange::binance::auth::generate_signature};
use rotom_data::shared::de::de_str;

/*----- */
// Binance Cancel Order - Single
/*----- */
// Cancel order for a given client_order_id
#[derive(Debug, Serialize)]
pub struct BinanceCancelOrderParams {
    #[serde(rename(serialize = "origClientOrderId"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_client_order_id: Option<String>,
    pub signature: Option<String>,
    pub symbol: String,
    pub timestamp: i64,
}

impl BinanceCancelOrderParams {
    pub fn new(orig_client_order_id: String, symbol: String) -> Result<Self, RequestBuildError> {
        BinanceCancelOrderParamsBuilder::new()
            .orig_client_order_id(orig_client_order_id)
            .symbol(symbol)
            .sign()
            .build()
    }
}

/*----- */
// Impl RestRequest for Binance Cancel Order
/*----- */
impl RestRequest for BinanceCancelOrderParams {
    type Response = BinanceCancelOrderResponse;
    type QueryParams = Self;
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/order")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::DELETE
    }

    fn query_params(&self) -> Option<&Self> {
        Some(self)
    }
}

/*----- */
// Binance Cancel Order - All
/*----- */
// Cancel order for a given symbol, i.e., cancels all open order for
// OPUSDT. We use the same builder to build the single and cancel all
// request as only one field is different. We need two separate struct
// as the path is different in when impl RestRequest. Hence, the from
// trait is implemented to convert one to the other.
#[derive(Debug, Serialize)]
pub struct BinanceCancelAllOrderParams {
    pub signature: Option<String>,
    pub symbol: String,
    pub timestamp: i64,
}

impl From<BinanceCancelOrderParams> for BinanceCancelAllOrderParams {
    fn from(cancel_order: BinanceCancelOrderParams) -> Self {
        Self {
            signature: cancel_order.signature,
            symbol: cancel_order.symbol,
            timestamp: cancel_order.timestamp,
        }
    }
}

impl BinanceCancelAllOrderParams {
    pub fn new(symbol: String) -> Result<Self, RequestBuildError> {
        Ok(BinanceCancelAllOrderParams::from(
            BinanceCancelOrderParamsBuilder::new()
                .symbol(symbol)
                .sign()
                .build()?,
        ))
    }
}

/*----- */
// Impl RestRequest for Binance Cancel Order
/*----- */
impl RestRequest for BinanceCancelAllOrderParams {
    type Response = Vec<BinanceCancelOrderResponse>;
    type QueryParams = Self;
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/openOrders")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::DELETE
    }

    fn query_params(&self) -> Option<&Self> {
        Some(self)
    }
}

/*----- */
// Binance Cancel Order Builder
/*----- */
#[derive(Debug, Serialize)]
pub struct BinanceCancelOrderParamsBuilder {
    #[serde(rename(serialize = "origClientOrderId"))]
    pub orig_client_order_id: Option<String>,
    pub signature: Option<String>,
    pub symbol: Option<String>,
    pub timestamp: i64,
}

impl Default for BinanceCancelOrderParamsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BinanceCancelOrderParamsBuilder {
    pub fn new() -> Self {
        Self {
            orig_client_order_id: None,
            signature: None,
            symbol: None,
            timestamp: Utc::now().timestamp_millis(),
        }
    }

    pub fn orig_client_order_id(self, orig_client_order_id: String) -> Self {
        Self {
            orig_client_order_id: Some(orig_client_order_id),
            ..self
        }
    }

    pub fn symbol(self, symbol: String) -> Self {
        Self {
            symbol: Some(symbol),
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

    pub fn build(self) -> Result<BinanceCancelOrderParams, RequestBuildError> {
        Ok(BinanceCancelOrderParams {
            orig_client_order_id: self.orig_client_order_id,
            symbol: self.symbol.ok_or(RequestBuildError::BuilderError {
                exchange: "Binance",
                request: "cancel order: symbol",
            })?,
            signature: self.signature,
            timestamp: self.timestamp,
        })
    }
}

/*----- */
// Cancel Order Responses
/*----- */
#[derive(Debug, Deserialize)]
pub struct BinanceCancelOrderResponse {
    pub symbol: String,
    #[serde(alias = "origClientOrderId")]
    pub orig_client_order_id: String,
    #[serde(alias = "orderId")]
    pub order_id: u64,
    #[serde(alias = "orderListId")]
    pub order_list_id: i64,
    #[serde(alias = "clientOrderId")]
    pub client_order_id: String,
    #[serde(alias = "transactTime")]
    pub transact_time: u64,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str", alias = "origQty")]
    pub orig_qty: f64,
    #[serde(deserialize_with = "de_str", alias = "executedQty")]
    pub executed_qty: f64,
    #[serde(deserialize_with = "de_str", alias = "cummulativeQuoteQty")]
    pub cummulative_quote_qty: f64,
    pub status: BinanceOrderStatus,
    #[serde(alias = "timeInForce")]
    pub time_in_force: BinanceTimeInForce,
    pub r#type: String, // Change to binance type
    pub side: BinanceSide,
    #[serde(alias = "stopPrice")]
    pub stop_price: Option<String>, // TODO: deserialise into a f64
    #[serde(alias = "trailingDelta")]
    pub trailing_delta: Option<u64>,
    #[serde(alias = "icebergQty")]
    pub iceberg_qty: Option<String>, // TODO: deserialise into a f64
    #[serde(alias = "strategyId")]
    pub strategy_id: Option<u64>,
    #[serde(alias = "strategyType")]
    pub strategy_type: Option<u64>,
    #[serde(alias = "selfTradePreventionMode")]
    pub self_trade_prevention_mode: String,
}
