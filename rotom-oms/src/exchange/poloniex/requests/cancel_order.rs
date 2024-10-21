use std::borrow::Cow;

use rotom_data::protocols::http::rest_request::RestRequest;
use serde::Serialize;
use serde_json::Value;

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
    type Response = Value; // TODO
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
    type Response = Value; // todo
    type QueryParams = (); // todo
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
