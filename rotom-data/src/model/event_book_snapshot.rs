use crate::assets::level::Level;

use super::{EventKind, SubKind};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct OrderBookSnapshot;

impl SubKind for OrderBookSnapshot {
    const EVENTKIND: EventKind = EventKind::OrderBook;
    type Event = EventOrderBookSnapshot;
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct EventOrderBookSnapshot {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}
