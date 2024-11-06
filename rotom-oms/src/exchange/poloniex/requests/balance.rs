use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use rotom_data::protocols::http::rest_request::RestRequest;
use rotom_data::shared::de::de_str;

/*----- */
// Poloniex Balance
/*----- */
#[derive(Debug, Default, Serialize)]
pub struct PoloniexBalance;

impl RestRequest for PoloniexBalance {
    type Response = Vec<PoloniexBalanceResponse>;
    type QueryParams = ();
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/accounts/balances")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::GET
    }
}

/*----- */
// Poloniex Balance - Response
/*----- */
#[derive(Debug, Default, Deserialize)]
pub struct PoloniexBalanceResponse {
    #[serde(alias = "accountId", deserialize_with = "de_str")]
    pub account_id: u64,
    #[serde(alias = "accountType")]
    pub account_type: String,
    pub balances: Vec<PoloniexBalanceResponseData>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PoloniexBalanceResponseData {
    #[serde(alias = "currencyId", deserialize_with = "de_str")]
    pub currency_id: u64,
    pub currency: String,
    #[serde(deserialize_with = "de_str")]
    pub available: f64,
    #[serde(deserialize_with = "de_str")]
    pub hold: f64,
}
