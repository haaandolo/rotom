use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use rotom_data::{
    protocols::http::{request_builder::Authenticator, rest_request::RestRequest},
    shared::de::de_str,
};

use crate::exchange::{binance::request_builder::BinanceAuthParams, errors::RequestBuildError};

/*----- */
// Binance Balance
/*----- */
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BinanceBalance {
    #[serde(rename(serialize = "omitZeroBalances"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omit_zero_balances: Option<bool>,
    #[serde(rename(serialize = "recWindow"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rec_window: Option<u64>, // value cannot be > 60_000
    pub signature: Option<String>,
    pub timestamp: i64,
}

impl BinanceBalance {
    pub fn new() -> Result<Self, RequestBuildError> {
        BinanceBalanceBuilder::default()
            .omit_zero_balances(true)
            .sign()
            .build()
    }
}

impl RestRequest for BinanceBalance {
    type Response = BinanceBalanceResponse;
    type QueryParams = Self;
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/account")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::GET
    }

    fn query_params(&self) -> Option<&Self> {
        Some(self)
    }
}

/*----- */
// Binance Balance Builder
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceBalanceBuilder {
    #[serde(rename(serialize = "omitZeroBalances"))]
    pub omit_zero_balances: Option<bool>,
    #[serde(rename(serialize = "recWindow"))]
    pub rec_window: Option<u64>, // value cannot be > 60_000
    pub timestamp: i64,
    pub signature: Option<String>,
}

impl Default for BinanceBalanceBuilder {
    fn default() -> Self {
        Self {
            omit_zero_balances: None,
            rec_window: None,
            timestamp: Utc::now().timestamp_millis(),
            signature: None,
        }
    }
}

impl BinanceBalanceBuilder {
    pub fn omit_zero_balances(self, omit_zero_balances: bool) -> Self {
        Self {
            omit_zero_balances: Some(omit_zero_balances),
            ..self
        }
    }

    pub fn rec_window(self, rec_window: u64) -> Self {
        Self {
            rec_window: Some(rec_window),
            ..self
        }
    }

    pub fn sign(self) -> Self {
        let signature = BinanceAuthParams::generate_signature(
            serde_urlencoded::to_string(&self).unwrap_or_default(),
        );
        Self {
            signature: Some(signature),
            ..self
        }
    }

    pub fn build(self) -> Result<BinanceBalance, RequestBuildError> {
        Ok(BinanceBalance {
            omit_zero_balances: self.omit_zero_balances,
            rec_window: self.rec_window,
            signature: self.signature,
            timestamp: self.timestamp,
        })
    }
}

/*----- */
// Binance Balance - Response
/*----- */
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceBalanceResponse {
    pub maker_commission: f64,
    pub taker_commission: f64,
    pub buyer_commission: f64,
    pub seller_commission: f64,
    pub commission_rates: BinanceBalanceResponseCommisionData,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub brokered: bool,
    pub require_self_trade_prevention: bool,
    pub prevent_sor: bool,
    pub update_time: u64,
    pub account_type: String,
    pub balances: Vec<BinanceBalanceResponseData>,
    pub permission: Vec<String>,
    pub uuid: u64,
}

#[derive(Debug, Default, Deserialize)]
pub struct BinanceBalanceResponseCommisionData {
    #[serde(deserialize_with = "de_str")]
    pub maker: f64,
    #[serde(deserialize_with = "de_str")]
    pub taker: f64,
    #[serde(deserialize_with = "de_str")]
    pub buyer: f64,
    #[serde(deserialize_with = "de_str")]
    pub seller: f64,
}

#[derive(Debug, Default, Deserialize)]
pub struct BinanceBalanceResponseData {
    pub asset: String,
    #[serde(deserialize_with = "de_str")]
    pub fee: f64,
    #[serde(deserialize_with = "de_str")]
    pub locked: f64,
}
