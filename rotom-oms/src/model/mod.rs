pub mod account_data;
pub mod balance;
pub mod order;

use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    fmt::{Display, Formatter},
};

/*----- */
// Client Order Id
/*----- */
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct ClientOrderId(pub String); // can be smolstr probs

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

impl<T> From<T> for Side
where
    T: Borrow<Decision>,
{
    fn from(decision: T) -> Self {
        match decision.borrow() {
            Decision::Long => Side::Buy,
            Decision::CloseLong => Side::Sell,
            Decision::Short => Side::Sell,
            Decision::CloseShort => Side::Buy,
        }
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Side::Buy => "buy",
                Side::Sell => "sell",
            }
        )
    }
}

/*----- */
// Order Kind
/*----- */
#[derive(Debug, Eq, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum OrderKind {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker,
}

impl AsRef<str> for OrderKind {
    fn as_ref(&self) -> &str {
        match self {
            OrderKind::Market => "market",
            OrderKind::Limit => "limit",
            OrderKind::LimitMaker => "limit maker",
            OrderKind::StopLoss => "stop loss",
            OrderKind::StopLossLimit => "stop loss limit",
            OrderKind::TakeProfit => "take profit",
            OrderKind::TakeProfitLimit => "take profit limit",
        }
    }
}

impl Display for OrderKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderKind::Market => "market",
                OrderKind::Limit => "limit",
                OrderKind::LimitMaker => "limit maker",
                OrderKind::StopLoss => "stop loss",
                OrderKind::StopLossLimit => "stop loss limit",
                OrderKind::TakeProfit => "take profit",
                OrderKind::TakeProfitLimit => "take profit limit",
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::{Decision, Side};

    #[test]
    fn test_side_from_owned_and_ref() {
        let buy = Side::from(Decision::Long);
        assert_eq!(buy, Side::Buy);

        let buy_ref = Side::from(&Decision::Long);
        assert_eq!(buy_ref, Side::Buy);
    }
}
