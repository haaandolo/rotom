use serde::Deserialize;


use super::{level::Level, SubKind};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct Trades;

impl SubKind for Trades {
    type Event = Trade;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize)]
pub struct Trade {
    trade: Level,
    is_buy: bool,
}

impl Trade {
    pub fn new(trade: Level, is_buy: bool) -> Self {
        Self { trade, is_buy }
    }
}
