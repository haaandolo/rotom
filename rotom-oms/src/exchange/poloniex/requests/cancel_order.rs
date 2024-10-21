use std::borrow::Cow;

use rotom_data::protocols::http::rest_request::RestRequest;
use serde::Serialize;
use serde_json::Value;

/*----- */
// Poloniex New Order
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
