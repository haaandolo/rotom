use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use rotom_data::shared::de::de_str;
use rotom_data::{
    protocols::http::rest_request::RestRequest, shared::subscription_models::ExchangeId,
};

use crate::portfolio::{AssetBalance, Balance};

/*----- */
// Poloniex Balance
/*----- */
#[derive(Debug, Default, Serialize)]
pub struct PoloniexBalance;

impl RestRequest for PoloniexBalance {
    type Response = PoloniexBalanceResponse;
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
pub struct PoloniexBalanceResponse(Vec<PoloniexBalanceResponseData>);

#[derive(Debug, Default, Deserialize)]
pub struct PoloniexBalanceResponseData {
    #[serde(alias = "accountId", deserialize_with = "de_str")]
    pub account_id: u64,
    #[serde(alias = "accountType")]
    pub account_type: String,
    pub balances: Vec<PoloniexBalanceResponseVec>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PoloniexBalanceResponseVec {
    #[serde(alias = "currencyId", deserialize_with = "de_str")]
    pub currency_id: u64,
    pub currency: String,
    #[serde(deserialize_with = "de_str")]
    pub available: f64,
    #[serde(deserialize_with = "de_str")]
    pub hold: f64,
}

/*----- */
// Impl balance from trait
/*----- */
impl From<PoloniexBalanceResponseVec> for Balance {
    fn from(balance: PoloniexBalanceResponseVec) -> Self {
        Self {
            time: Utc::now(),
            total: balance.available,
            available: 0.0,
        }
    }
}

impl From<PoloniexBalanceResponse> for Vec<AssetBalance> {
    fn from(polo_balances: PoloniexBalanceResponse) -> Self {
        let mut asset_balances = Vec::new();
        for polo_balance in polo_balances.0.into_iter() {
            for mut sub_balance in polo_balance.balances.into_iter() {
                asset_balances.push(AssetBalance {
                    asset: std::mem::take(&mut sub_balance.currency),
                    exchange: ExchangeId::PoloniexSpot,
                    balance: Balance::from(sub_balance),
                })
            }
        }
        asset_balances
    }
}
