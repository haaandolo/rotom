pub mod balance;
pub mod cancel_order;
pub mod new_order;
pub mod account_data;
pub mod wallet_transfer;
pub mod ws_auth;

use rotom_data::shared::subscription_models::Instrument;
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PoloniexTimeInForce {
    GTC,
    IOC,
    FOK,
}

impl AsRef<str> for PoloniexTimeInForce {
    fn as_ref(&self) -> &str {
        match self {
            PoloniexTimeInForce::GTC => "GTC",
            PoloniexTimeInForce::IOC => "IOC",
            PoloniexTimeInForce::FOK => "FOK",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PoloniexSide {
    BUY,
    SELL,
}

impl AsRef<str> for PoloniexSide {
    fn as_ref(&self) -> &str {
        match self {
            PoloniexSide::BUY => "buy",
            PoloniexSide::SELL => "sell",
        }
    }
}

impl From<Decision> for PoloniexSide {
    fn from(decision: Decision) -> Self {
        match decision {
            Decision::Long => PoloniexSide::BUY,
            Decision::CloseLong => PoloniexSide::SELL,
            Decision::Short => PoloniexSide::SELL,
            Decision::CloseShort => PoloniexSide::BUY,
        }
    }
}

#[derive(Debug)]
pub struct PoloniexSymbol(pub String);

impl From<&Instrument> for PoloniexSymbol {
    fn from(instrument: &Instrument) -> PoloniexSymbol {
        PoloniexSymbol(format!("{}_{}", instrument.base, instrument.quote))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PoloniexOrderType {
    Market,
    Limit,
    LimitMaker,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum PoloniexOrderStatus {
    New,
    PartiallyFilled,
    Filled,
    PendingCancel,
    PartiallyCanceled,
    Canceled,
    Failed,
}
