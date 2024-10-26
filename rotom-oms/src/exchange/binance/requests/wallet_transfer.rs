use chrono::Utc;
use rotom_data::protocols::http::{request_builder::Authenticator, rest_request::RestRequest};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::exchange::{binance::request_builder::BinanceAuthParams, errors::RequestBuildError};

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
    pub fn new(coin: String, wallet_address: String, amount: f64) -> Result<Self, RequestBuildError> {
        Self::builder()
            .coin(coin)
            .amount(amount) // todo
            .address(wallet_address)
            .sign()
            .build()
    }

    pub fn builder() -> BinanceWalletTransferBuilder {
        BinanceWalletTransferBuilder::new()
    }
}

impl RestRequest for BinanceWalletTransfer {
    type Response = BinanceWalletTransferResponse;
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
        let signature =
            BinanceAuthParams::generate_signature(serde_urlencoded::to_string(&self).unwrap()); // TODO
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

/*----- */
// Wallet Transfer Reponse
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceWalletTransferResponse {
    id: String,
}
