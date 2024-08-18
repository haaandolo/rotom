use chrono::{DateTime, Utc};

use crate::assets::level::Level;

use super::SubKind;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct OrderBookL2;

impl SubKind for OrderBookL2 {
    type Event = EventOrderBook;
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct EventOrderBook {
    pub last_update_time: DateTime<Utc>,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

impl EventOrderBook {
    pub fn new(last_update_time: DateTime<Utc>, bids: Vec<Level>, asks: Vec<Level>) -> Self {
        Self {
            last_update_time,
            bids,
            asks,
        }
    }

    #[inline]
    pub fn weighted_midprice(&self) -> Option<f64> {
        if let (Some(best_bid), Some(best_ask)) = (self.bids.first(), self.asks.first()) {
            let num = best_bid.size * best_ask.price + best_bid.price * best_ask.size;
            let den = best_bid.size + best_ask.size;
            return Some(num / den);
        }

        None
    }

    #[inline]
    pub fn midprice(&self) -> Option<f64> {
        if let (Some(best_bid), Some(best_ask)) = (self.bids.first(), self.asks.first()) {
            return Some((best_bid.price + best_ask.price) / 2.0);
        }

        None
    }
}
