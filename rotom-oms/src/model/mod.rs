pub mod balance;
pub mod execution_request;
pub mod account;
pub mod order;

use chrono::{DateTime, Utc};
use rand::seq::SliceRandom;
use rotom_data::shared::subscription_models::ExchangeId;
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    fmt::{Display, Formatter},
};

use crate::execution_manager::builder::TraderId;

/*----- */
// Generic order type
/*----- */
#[derive(Debug, Clone)]
pub struct Order<RequestResponse> {
    pub trader_id: TraderId,
    pub exchange: ExchangeId,
    pub requested_time: DateTime<Utc>,
    pub request_response: RequestResponse,
}

/*----- */
// Client Order Id
/*----- */
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct ClientOrderId<T = String>(pub T); // can be smolstr probs

impl ClientOrderId<String> {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }

    pub fn random() -> Self {
        const LEN_URL_SAFE_SYMBOLS: usize = 64;
        const URL_SAFE_SYMBOLS: [char; LEN_URL_SAFE_SYMBOLS] = [
            '_', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e',
            'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
            'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
            'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ];

        const LEN_RANDOM_ID: usize = 23; // Kept same length for compatibility
        let mut thread_rng = rand::thread_rng();
        let random_utf8: [u8; LEN_RANDOM_ID] = std::array::from_fn(|_| {
            let symbol = URL_SAFE_SYMBOLS
                .choose(&mut thread_rng)
                .expect("URL_SAFE_SYMBOLS slice is not empty");
            *symbol as u8
        });

        let random_utf8_str =
            std::str::from_utf8(&random_utf8).expect("URL_SAFE_SYMBOLS are valid utf8");
        Self(random_utf8_str.to_string())
    }
}

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
}

impl AsRef<str> for OrderKind {
    fn as_ref(&self) -> &str {
        match self {
            OrderKind::Market => "market",
            OrderKind::Limit => "limit",
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
