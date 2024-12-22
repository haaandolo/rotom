use std::borrow::Cow;

use rotom_data::protocols::http::rest_request::RestRequest;
use serde::{Deserialize, Serialize};

/*----- */
// Poloniex Wallet Transfer
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct PoloniexWalletTransfer {
    pub currency: String,
    pub amount: f64,
    pub address: String,
}

impl PoloniexWalletTransfer {
    pub fn new(mut coin: String, network: Option<String>, amount: f64, address: String) -> Self {
        // If network is provided push onto coin field: https://api-docs.poloniex.com/spot/api/private/wallet
        if let Some(network) = network {
            coin.push_str(&network);
        }

        Self {
            currency: coin.to_uppercase(),
            amount,
            address,
        }
    }
}

impl RestRequest for PoloniexWalletTransfer {
    type Response = PoloniexWalletTransferResponse;
    type QueryParams = ();
    type Body = Self;

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/wallets/withdraw")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::POST
    }

    fn body(&self) -> Option<&Self::Body> {
        Some(self)
    }
}

/*----- */
// Poloniex Wallet Transfer Response
/*----- */
#[derive(Debug, Deserialize)]
pub struct PoloniexWalletTransferResponse {
    #[serde(alias = "withdrawalRequestsId")]
    pub withdrawal_request_id: u64,
}
