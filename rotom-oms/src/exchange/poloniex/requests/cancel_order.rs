use std::borrow::Cow;

use rotom_data::protocols::http::rest_request::RestRequest;
use serde::{Deserialize, Serialize};

use super::PoloniexOrderStatus;
use rotom_data::shared::de::de_str;

/*----- */
// Poloniex Cancel Order
/*----- */
#[derive(Debug, Serialize)]
pub struct PoloniexCancelOrder {
    id: String,
}

impl PoloniexCancelOrder {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl RestRequest for PoloniexCancelOrder {
    type Response = Vec<PoloniexCancelOrderResponse>;
    type QueryParams = ();
    type Body = Self;

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/orders")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::DELETE
    }

    fn body(&self) -> Option<&Self::Body> {
        Some(self)
    }
}

/*----- */
// Poloniex Cancel Order - All
/*----- */
#[derive(Debug, Serialize)]
pub struct PoloniexCancelAllOrder {
    symbols: Vec<String>,
    #[serde(rename(serialize = "accountTypes"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    account_types: Option<Vec<String>>,
}

impl PoloniexCancelAllOrder {
    pub fn new(symbol: String) -> Self {
        Self {
            symbols: vec![symbol],
            account_types: None,
        }
    }
}

impl RestRequest for PoloniexCancelAllOrder {
    type Response = Vec<PoloniexCancelOrderResponse>;
    type QueryParams = ();
    type Body = Self;

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/orders")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::DELETE
    }

    fn body(&self) -> Option<&Self::Body> {
        Some(self)
    }
}

/*----- */
// Poloniex Cancel Order Response
/*----- */
#[derive(Debug, Deserialize)]
pub struct PoloniexCancelOrderResponse {
    #[serde(deserialize_with = "de_str")]
    #[serde(alias = "orderId")]
    pub order_id: u64,
    #[serde(alias = "clientOrderId")]
    pub client_order_id: String,
    pub state: PoloniexOrderStatus,
    pub code: u64,
    pub message: String,
}