use std::borrow::Cow;

use rotom_data::protocols::http::rest_request::RestRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/*----- */
// Poloniex Wallet Transfer
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct PoloniexWalletTransfer {
    pub coin: String,
    pub network: Option<String>,
    pub amount: f64,
    pub address: String,
    #[serde(rename(serialize = "addressTag"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_tag: Option<String>,
    #[serde(rename(serialize = "allowBorrow"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_borrow: Option<bool>,
}

impl PoloniexWalletTransfer {
    pub fn new(coin: String, network: Option<String>, amount: f64, address: String) -> Self {
        Self {
            coin,
            network,
            amount,
            address,
            address_tag: None,
            allow_borrow: None,
        }
    }
}

impl RestRequest for PoloniexWalletTransfer {
    type Response = Value; // todo
    type QueryParams = ();
    type Body = Self;

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/v2/wallets/withdraw")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::POST
    }

    fn body(&self) -> Option<&Self::Body> {
        Some(self)
    }
}
