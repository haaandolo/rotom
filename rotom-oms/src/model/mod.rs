pub mod balance;
pub mod order;
pub mod trade;
use std::fmt::{Display, Formatter};

use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};

/*----- */
// Client Order Id
/*----- */
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct ClientOrderId(pub String); // todo: change to UUid

/*----- */
// Order Id
/*----- */
#[derive(Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct OrderId(pub String); // can be smolstr

/*----- */
// Side
/*----- */
// todo side should be in intergration crate, which isnt made yet
#[derive(
    Copy, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize,
)]
pub enum Side {
    #[serde(alias = "buy", alias = "BUY", alias = "b")]
    #[default]
    Buy,
    #[serde(alias = "sell", alias = "SELL", alias = "s")]
    Sell,
}

impl From<Decision> for Side {
    fn from(decision: Decision) -> Self {
        match decision {
            Decision::Long => Side::Buy,
            Decision::CloseLong => Side::Sell,
            Decision::Short => Side::Sell,
            Decision::CloseShort => Side::Buy,
        }
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "buy"),
            Side::Sell => write!(f, "sell"),
        }
    }
}

/*----- */
// Order Kind
/*----- */
pub enum OrderKind {
    Market,
    Limit,
}

impl Display for OrderKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderKind::Market => "market",
                OrderKind::Limit => "limit",
            }
        )
    }
}

/*----- */
// State
/*----- */
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum State {
    New,
    PartiallyFilled,
    Filled,
    PendingCancel,
    PartiallyCanceled,
    Canceled,
    Failed,
    Replaced,
    Rejected,
    Trade,
    Expired,
    TradePrevention,
    PendingNew,
    ExpiredInMatch,
}
