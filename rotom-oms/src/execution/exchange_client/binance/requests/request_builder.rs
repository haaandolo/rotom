use serde::Serialize;
use uuid::Uuid;

use crate::{execution::error::RequestBuildError, portfolio::OrderEvent};

use super::{
    cancel_order::BinanceCancelOrderParams, new_order::BinanceNewOrderParams,
    wallet_transfer::BinanceWalletTransfer,
};

/*----- */
// Binance Requests
/*----- */
#[derive(Debug, Serialize, Default)]
pub struct BinanceOrder<RequestParams> {
    id: Uuid,
    method: &'static str,
    pub params: RequestParams,
}

#[derive(Debug, Serialize, Default)]
pub struct BinanceRequest;

impl BinanceRequest {
    pub fn new_order(
        order_event: &OrderEvent,
    ) -> Result<BinanceOrder<BinanceNewOrderParams>, RequestBuildError> {
        BinanceRequestBuilder::new()
            .method("order.place")
            .params(BinanceNewOrderParams::new(order_event))
            .build()
    }

    pub fn cancel_order(
        orig_client_order_id: String,
        symbol: String,
    ) -> Result<BinanceOrder<BinanceCancelOrderParams>, RequestBuildError> {
        BinanceRequestBuilder::new()
            .method("order.cancel")
            .params(BinanceCancelOrderParams::cancel_order(
                orig_client_order_id,
                symbol,
            )?)
            .build()
    }

    pub fn cancel_order_all(
        symbol: String,
    ) -> Result<BinanceOrder<BinanceCancelOrderParams>, RequestBuildError> {
        BinanceRequestBuilder::new()
            .method("openOrders.cancelAll")
            .params(BinanceCancelOrderParams::cancel_order_all(symbol)?)
            .build()
    }

    pub fn wallet_transfer(
        coin: String,
        wallet_address: String,
    ) -> Result<String, RequestBuildError> {
        Ok(BinanceWalletTransfer::builder()
            .coin(coin)
            .amount(1.01)
            .address(wallet_address)
            .sign()
            .build()?
            .query_param())
    }
}

/*----- */
// Binance Requests Builder
/*----- */
#[derive(Debug, Serialize)]
pub struct BinanceRequestBuilder<RequestParams> {
    id: Uuid,
    method: Option<&'static str>,
    pub params: Option<RequestParams>,
}

impl<RequestParams> Default for BinanceRequestBuilder<RequestParams> {
    fn default() -> Self {
        Self::new()
    }
}

impl<RequestParams> BinanceRequestBuilder<RequestParams> {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            method: None,
            params: None,
        }
    }

    pub fn method(self, method: &'static str) -> Self {
        Self {
            method: Some(method),
            ..self
        }
    }

    pub fn params(self, params: RequestParams) -> Self {
        Self {
            params: Some(params),
            ..self
        }
    }

    pub fn build(self) -> Result<BinanceOrder<RequestParams>, RequestBuildError> {
        Ok(BinanceOrder {
            id: self.id,
            method: self.method.ok_or(RequestBuildError::BuilderError {
                exchange: "Binance",
                request: "binance request builder: method",
            })?,
            params: self.params.ok_or(RequestBuildError::BuilderError {
                exchange: "Binance",
                request: "binance request builder: params",
            })?,
        })
    }
}
