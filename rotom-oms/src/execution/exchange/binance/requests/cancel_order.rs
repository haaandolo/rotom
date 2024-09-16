use chrono::Utc;
use serde::Serialize;

use crate::execution::{
    error::RequestBuildError, exchange::binance::auth::generate_signature,
};

/*----- */
// Binance Cancel Order
/*----- */
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
    pub fn cancel_order(
        orig_client_order_id: String,
        symbol: String,
    ) -> Result<Self, RequestBuildError> {
        BinanceCancelOrderParamsBuilder::new()
            .orig_client_order_id(orig_client_order_id)
            .symbol(symbol)
            .sign()
            .build()
    }

    pub fn cancel_order_all(symbol: String) -> Result<Self, RequestBuildError> {
        BinanceCancelOrderParamsBuilder::new()
            .symbol(symbol)
            .sign()
            .build()
    }

    pub fn query_param(&self) -> String {
        serde_urlencoded::to_string(self).unwrap()
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
