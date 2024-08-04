use serde::Deserialize;

use super::{level::Level, SubKind};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Default)]
pub struct OrderBookL2;

impl SubKind for OrderBookL2 {
    type Event = EventOrderBook;
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize)]
pub struct EventOrderBook {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

impl EventOrderBook {
    pub fn new(bids: Vec<Level>, asks: Vec<Level>) -> Self {
        Self { bids, asks }
    }
}
