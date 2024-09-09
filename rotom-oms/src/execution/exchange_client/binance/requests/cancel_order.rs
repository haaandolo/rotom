use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::execution::exchange_client::{binance::auth::BinanceAuthParams, ParamString};

/*----- */
// Binance Cancel Order - for single order
/*----- */
#[derive(Debug, Serialize)]
pub struct BinanceCancelOrder {
    id: Uuid,
    method: &'static str,
    pub params: BinanceCancelOrderParams,
}

#[derive(Debug, Serialize)]
pub struct BinanceCancelOrderParams {
    #[serde(rename(serialize = "apiKey"))]
    pub api_key: &'static str,
    #[serde(rename(serialize = "origClientOrderId"))]
    pub orig_client_order_id: String,
    pub signature: Option<String>,
    pub symbol: String,
    pub timestamp: i64,
}

impl BinanceCancelOrder {
    pub fn new(orig_client_order_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            method: "order.cancel",
            params: BinanceCancelOrderParams {
                api_key: BinanceAuthParams::KEY,
                orig_client_order_id,
                signature: None,
                symbol: "OPUSDT".to_string(), // TODO
                timestamp: Utc::now().timestamp_millis(),
            },
        }
    }

    pub fn get_query_param(&self) -> ParamString {
        ParamString(serde_urlencoded::to_string(&self.params).unwrap_or_default())
    }
}

/*----- */
// Binance Cancel Order All - cancel all order for given asset
/*----- */
#[derive(Debug, Serialize)]
pub struct BinanceCancelAllOrder {
    id: Uuid,
    method: &'static str,
    pub params: BinanceCancelAllOrderParams,
}

#[derive(Debug, Serialize)]
pub struct BinanceCancelAllOrderParams {
    #[serde(rename(serialize = "apiKey"))]
    pub api_key: &'static str,
    pub signature: Option<String>,
    pub symbol: String,
    pub timestamp: i64,
}

impl BinanceCancelAllOrder {
    pub fn new(symbol: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            method: "openOrders.cancelAll",
            params: BinanceCancelAllOrderParams {
                api_key: BinanceAuthParams::KEY,
                signature: None,
                symbol,
                timestamp: Utc::now().timestamp_millis(),
            },
        }
    }

    pub fn get_query_param(&self) -> ParamString {
        ParamString(serde_urlencoded::to_string(&self.params).unwrap_or_default())
    }
}
