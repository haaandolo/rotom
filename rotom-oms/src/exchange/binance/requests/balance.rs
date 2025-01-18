use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use rotom_data::{
    protocols::http::{request_builder::Authenticator, rest_request::RestRequest},
    shared::{de::de_str, subscription_models::ExchangeId},
};

use crate::{
    exchange::{binance::request_builder::BinanceAuthParams, errors::RequestBuildError},
    model::{execution_response::AccountDataBalance, balance::Balance},
};

/*----- */
// Binance Balance
/*----- */
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BinanceBalance {
    #[serde(
        rename(serialize = "omitZeroBalances"),
        skip_serializing_if = "Option::is_none"
    )]
    pub omit_zero_balances: Option<bool>,
    #[serde(
        rename(serialize = "recWindow"),
        skip_serializing_if = "Option::is_none"
    )]
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
pub struct BinanceBalanceResponse {
    #[serde(alias = "accountType")]
    pub account_type: String,
    pub balances: Vec<BinanceBalanceResponseData>,
    pub brokered: bool,
    #[serde(alias = "buyerCommission")]
    pub buyer_commission: f64,
    #[serde(alias = "canDeposit")]
    pub can_deposit: bool,
    #[serde(alias = "canTrade")]
    pub can_trade: bool,
    #[serde(alias = "canWithdraw")]
    pub can_withdraw: bool,
    #[serde(alias = "commissionRates")]
    pub commission_rates: BinanceBalanceCommisionRates,
    #[serde(alias = "makerCommission")]
    pub maker_commission: u64,
    pub permissions: Vec<String>,
    #[serde(alias = "preventSor")]
    pub prevent_sor: bool,
    #[serde(alias = "requireSelfTradePrevention")]
    pub require_self_trade_prevention: bool,
    #[serde(alias = "sellerCommission")]
    pub seller_commission: u64,
    #[serde(alias = "takerCommission")]
    pub taker_commission: u64,
    pub uid: u64,
    #[serde(alias = "updateTime")]
    pub update_time: u64,
}

#[derive(Debug, Default, Deserialize)]
pub struct BinanceBalanceCommisionRates {
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
    pub free: f64,
    #[serde(deserialize_with = "de_str")]
    pub locked: f64,
}

/*----- */
// Impl balance from trait
/*----- */
impl From<BinanceBalanceResponseData> for Balance {
    fn from(balance: BinanceBalanceResponseData) -> Self {
        Self {
            total: balance.free,
            available: 0.0,
        }
    }
}

impl From<BinanceBalanceResponse> for Vec<AccountDataBalance> {
    fn from(bin_balance: BinanceBalanceResponse) -> Self {
        bin_balance
            .balances
            .into_iter()
            .map(|mut balance| AccountDataBalance {
                asset: std::mem::take(&mut balance.asset),
                exchange: ExchangeId::BinanceSpot,
                balance: Balance::from(balance),
            })
            .collect()
    }
}
