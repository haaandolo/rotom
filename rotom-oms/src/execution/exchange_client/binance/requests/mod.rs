pub mod cancel_order;
pub mod new_order;
pub mod responses;

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
