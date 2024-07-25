use serde::Deserialize;

use super::{level::Level, SubKind};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct OrderBookL2;

impl SubKind for OrderBookL2 {
    type Event = EventOrderBook;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize)]
pub struct EventOrderBook {
    pub bids: Option<Vec<Level>>,
    pub asks: Option<Vec<Level>>,
}

impl EventOrderBook {
    pub fn new(bids: Option<Vec<Level>>, asks: Option<Vec<Level>>) -> Self {
        Self { bids, asks }
    }
}
