use serde::Deserialize;

use super::{level::Level, SubKind};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct Trades;

impl SubKind for Trades {
    type Event = EventTrade;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize)]
pub struct EventTrade {
    trade: Level,
    is_buy: bool,
}

impl EventTrade {
    pub fn new(trade: Level, is_buy: bool) -> Self {
        Self { trade, is_buy }
    }
}
