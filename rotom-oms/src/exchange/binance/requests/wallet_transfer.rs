use std::borrow::Cow;

use chrono::Utc;
use rotom_data::protocols::http::rest_request::RestRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::exchange::{binance::auth::generate_signature, errors::RequestBuildError};

/*----- */
// Binance Wallet Transfer
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceWalletTransfer {
    pub coin: String,
    pub amount: f64,
    pub timestamp: i64,
    pub address: String,
    pub signature: String,
}

impl BinanceWalletTransfer {
    pub fn new(coin: String, wallet_address: String) -> Result<Self, RequestBuildError> {
        Self::builder()
            .coin(coin)
            .amount(1.01)
            .address(wallet_address)
            .sign()
            .build()
    }

    pub fn builder() -> BinanceWalletTransferBuilder {
        BinanceWalletTransferBuilder::new()
    }
}

/*----- */
// Impl RestRequest for Binance Wallet Transfer
/*----- */
impl RestRequest for BinanceWalletTransfer {
    type Response = Value;
    type QueryParams = Self;
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/sapi/v1/capital/withdraw/apply")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::POST
    }

    fn query_params(&self) -> Option<&Self> {
        Some(self)
    }
}

/*----- */
// Binance Wallet Builder
/*----- */
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct BinanceWalletTransferBuilder {
    pub coin: Option<String>,
    pub amount: Option<f64>,
    pub timestamp: i64,
    pub address: Option<String>,
    pub signature: Option<String>,
}

impl BinanceWalletTransferBuilder {
    pub fn new() -> BinanceWalletTransferBuilder {
        Self {
            coin: None,
            amount: None,
            timestamp: Utc::now().timestamp_millis(),
            address: None,
            signature: None,
        }
    }

    pub fn coin(self, coin: String) -> Self {
        Self {
            coin: Some(coin),
            ..self
        }
    }

    pub fn amount(self, amount: f64) -> Self {
        Self {
            amount: Some(amount),
            ..self
        }
    }

    pub fn address(self, address: String) -> Self {
        Self {
            address: Some(address),
            ..self
        }
    }

    pub fn sign(self) -> Self {
        let signature = generate_signature(serde_urlencoded::to_string(&self).unwrap()); // TODO
        Self {
            signature: Some(signature),
            ..self
        }
    }

    pub fn build(self) -> Result<BinanceWalletTransfer, RequestBuildError> {
        Ok(BinanceWalletTransfer {
            coin: self.coin.ok_or(RequestBuildError::BuilderError {
                exchange: "Binance",
                request: "wallet transfer: coin",
            })?,
            amount: self.amount.ok_or(RequestBuildError::BuilderError {
                exchange: "Binance",
                request: "wallet transfer: amount",
            })?,
            timestamp: self.timestamp,
            address: self.address.ok_or(RequestBuildError::BuilderError {
                exchange: "Binance",
                request: "wallet transfer: address",
            })?,
            signature: self.signature.ok_or(RequestBuildError::BuilderError {
                exchange: "Binance",
                request: "wallet transfer: signature",
            })?,
        })
    }
}
