pub mod account_data;
pub mod balance;
pub mod cancel_order;
pub mod listening_key;
pub mod new_order;
pub mod wallet_transfer;

use rotom_data::shared::de::de_str;
use rotom_data::shared::subscription_models::Instrument;
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};

/*----- */
// Binance Order Enums and Conversion Implementations
/*----- */
/*
GTC: good till cancelled, an order will be on the book unless the order is cancelled
IOC: immediate or cancel, an order will try to fill the order as much as it can before the order expires
FOK: fill or kill, an order will expire if the full order cannot be filled upon execution
*/
#[derive(Debug, Serialize, Deserialize)]
pub enum BinanceTimeInForce {
    GTC,
    IOC,
    FOK,
}

impl AsRef<str> for BinanceTimeInForce {
    fn as_ref(&self) -> &str {
        match self {
            BinanceTimeInForce::GTC => "GTC",
            BinanceTimeInForce::IOC => "IOC",
            BinanceTimeInForce::FOK => "FOK",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BinanceSide {
    BUY,
    SELL,
}

impl AsRef<str> for BinanceSide {
    fn as_ref(&self) -> &str {
        match self {
            BinanceSide::BUY => "BUY",
            BinanceSide::SELL => "SELL",
        }
    }
}

impl From<Decision> for BinanceSide {
    fn from(decision: Decision) -> Self {
        match decision {
            Decision::Long => BinanceSide::BUY,
            Decision::CloseLong => BinanceSide::SELL,
            Decision::Short => BinanceSide::SELL,
            Decision::CloseShort => BinanceSide::BUY,
        }
    }
}

#[derive(Debug)]
pub struct BinanceSymbol(pub String);

impl From<&Instrument> for BinanceSymbol {
    fn from(instrument: &Instrument) -> BinanceSymbol {
        BinanceSymbol(format!("{}{}", instrument.base, instrument.quote).to_uppercase())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum BinanceOrderStatus {
    New,
    Canceled,
    Rejected,
    Expired,
    PendingNew,
    PartiallyFilled,
    Filled,
    PendingCancel,
    ExpiredInMatch,
}

#[derive(Debug, Deserialize)]
pub struct RateLimit {
    #[serde(alias = "rateLimitType")]
    pub rate_limit_type: String,
    pub interval: String,
    #[serde(alias = "intervalNum")]
    pub interval_num: u32,
    pub limit: u32,
    pub count: u32,
}

#[derive(Debug, Deserialize)]
pub struct BinanceFill {
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub qty: f64,
    #[serde(deserialize_with = "de_str")]
    pub commission: f64,
    #[serde(alias = "commissionAsset")]
    pub commission_asset: String,
    #[serde(alias = "tradeId")]
    pub trade_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct BinanaceErrorResponse {
    pub code: i16,
    pub msg: String,
}

/*----- */
// Tests
/*----- */
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_error() {
        let response = r#"{
            "code": -1021,
            "msg": "Timestamp for this request is outside of the recvWindow."
        }"#;

        let response_de = serde_json::from_str::<BinanaceErrorResponse>(response);
        let mut _result = false;

        match response_de {
            Ok(_) => _result = true,
            Err(_) => _result = false,
        }

        assert!(_result)
    }
}
