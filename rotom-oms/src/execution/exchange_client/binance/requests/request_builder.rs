use serde::Serialize;
use uuid::Uuid;

use crate::{execution::error::RequestBuildError, portfolio::OrderEvent};

use super::new_order::BinanceNewOrderParams;

/*----- */
// Binance Requests
/*----- */
#[derive(Debug, Serialize, Default)]
pub struct BinanceRequest<RequestParams> {
    id: Uuid,
    method: &'static str,
    pub params: RequestParams,
}

impl<RequestParams> BinanceRequest<RequestParams> {
    pub fn new_order(
        order_event: &OrderEvent,
    ) -> Result<BinanceRequest<BinanceNewOrderParams>, RequestBuildError> {
        BinanceRequestBuilder::new()
            .method("order.place")
            .params(BinanceNewOrderParams::new(order_event))
            .build()
    }

    pub fn cancel_order() -> Self {
        unimplemented!()
    }

    pub fn cancel_order_all() -> Self {
        unimplemented!()
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

    pub fn build(self) -> Result<BinanceRequest<RequestParams>, RequestBuildError> {
        Ok(BinanceRequest {
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
