use std::{cmp::Ordering, collections::BTreeMap, fmt::Display};

use serde::Deserialize;

use crate::data::shared::de::de_str;

/*----- */
// Levels
/*----- */
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct Level {
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub size: f64,
}

impl Level {
    pub fn new(_price: f64, _size: f64) -> Self {
        Self {
            price: _price,
            size: _size,
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.price.partial_cmp(&other.price) {
            Some(Ordering::Equal) => self.size.partial_cmp(&other.size),
            other_order => other_order,
        }
    }
}

impl Ord for Level {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Level {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.size == other.size
    }
}

impl Eq for Level {}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} : {})", self.price, self.size)
    }
}

/*----- */
// Event
/*----- */
#[derive(PartialEq, Debug)]
pub struct Event {
    pub symbol: String,
    pub timestamp: u64,
    pub seq: u64,
    pub bids: Option<Vec<Level>>,
    pub asks: Option<Vec<Level>>,
    pub trade: Option<Level>,
    pub is_buy: Option<bool>,
}

impl Event {
    pub fn new(
        _symbol: String,
        _timestamp: u64,
        _seq: u64,
        _bids: Option<Vec<Level>>,
        _asks: Option<Vec<Level>>,
        _trade: Option<Level>,
        _is_buy: Option<bool>,
    ) -> Self {
        Self {
            symbol: _symbol,
            timestamp: _timestamp,
            seq: _seq,
            bids: _bids,
            asks: _asks,
            trade: _trade,
            is_buy: _is_buy,
        }
    }
}

fn price_ticks(price: f64, tick_size: f64) -> u64 {
    (price * tick_size) as u64
}

/*----- */
// Orderbook
/*----- */
#[derive(Debug, Default, Clone)]
pub struct Orderbook {
    best_bid: Option<Level>,
    best_ask: Option<Level>,
    bids: BTreeMap<u64, Level>,
    asks: BTreeMap<u64, Level>,
    last_updated: u64,
    last_sequence: u64,
    inv_tick_size: f64,
}

impl Orderbook {
    pub fn new(tick_size: f64) -> Self {
        Self {
            best_bid: None,
            best_ask: None,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_updated: 0,
            last_sequence: 0,
            inv_tick_size: 1.0 / tick_size,
        }
    }

    #[inline]
    pub fn process_raw(&mut self, event: Event) {
        self.process(event);
    }

    #[inline]
    pub fn process_stream_bbo_raw(
        &mut self,
        event: Event,
    ) -> Option<(Option<Level>, Option<Level>)> {
        self.process_stream_bbo(event)
    }

    #[inline]
    pub fn process(&mut self, event: Event) {
        // let seq = event.seq;
        // let timestamp = event.timestamp;
        if event.timestamp < self.last_updated || event.seq < self.last_sequence {
            return;
        }

        match event.trade {
            Some(_) => self.process_trade(event),
            None => self.process_lvl2(event),
        }

        // self.last_updated = timestamp;
        // self.last_sequence = seq;
    }

    #[inline]
    pub fn process_stream_bbo(&mut self, event: Event) -> Option<(Option<Level>, Option<Level>)> {
        let old_bid = self.best_bid;
        let old_ask = self.best_ask;

        self.process(event);

        let new_bid = self.best_bid;
        let new_ask = self.best_ask;

        if old_bid != new_bid || old_ask != new_ask {
            Some((new_bid, new_ask))
        } else {
            None
        }
    }

    #[inline]
    fn process_lvl2(&mut self, event: Event) {
        // Process bids
        if let Some(levels) = event.bids {
            levels.into_iter().for_each(|level| {
                let price_tick = price_ticks(level.price, self.inv_tick_size);
                if level.size == 0.0 {
                    if let Some(removed) = self.bids.remove(&price_tick) {
                        if let Some(best_bid) = self.best_bid {
                            if removed.price == best_bid.price {
                                self.best_bid = self.bids.values().next_back().cloned();
                            }
                        }
                    }
                } else {
                    self.bids
                        .entry(price_tick)
                        .and_modify(|e| e.size = level.size)
                        .or_insert(level);

                    let Some(best_bid) = self.best_bid else {
                        self.best_bid = Some(level);
                        return;
                    };

                    if level.price >= best_bid.price {
                        self.best_bid = Some(level);
                    }
                }
            })
        }

        // Process asks
        if let Some(levels) = event.asks {
            levels.into_iter().for_each(|level| {
                let price_tick = price_ticks(level.price, self.inv_tick_size);
                if level.size == 0.0 {
                    if let Some(removed) = self.asks.remove(&price_tick) {
                        if let Some(best_ask) = self.best_ask {
                            if removed.price == best_ask.price {
                                self.best_ask = self.asks.values().next().cloned();
                            }
                        }
                    }
                } else {
                    self.asks
                        .entry(price_tick)
                        .and_modify(|e| e.size = level.size)
                        .or_insert(level);

                    let Some(best_ask) = self.best_ask else {
                        self.best_ask = Some(level);
                        return;
                    };

                    if level.price <= best_ask.price {
                        self.best_ask = Some(level)
                    }
                }
            })
        }

        self.last_updated = event.timestamp;
        self.last_sequence = event.seq;
    }

    #[inline]
    fn process_trade(&mut self, event: Event) {
        let buf = if event.is_buy == Some(true) {
            &mut self.bids
        } else {
            &mut self.asks
        };

        let trade_level = event.trade.unwrap();
        let price_ticks = price_ticks(trade_level.price, self.inv_tick_size);

        if let Some(level) = buf.get_mut(&price_ticks) {
            if trade_level.size >= level.size {
                buf.remove(&price_ticks);
            } else {
                level.size -= trade_level.size;
            }
        };

        self.last_updated = event.timestamp;
        self.last_sequence = event.seq;
    }

    pub fn best_bid(&self) -> Option<Level> {
        self.best_bid
    }

    pub fn best_ask(&self) -> Option<Level> {
        self.best_ask
    }

    #[inline]
    pub fn top_bids(&self, n: usize) -> Vec<Level> {
        self.bids.values().rev().take(n).cloned().collect()
    }

    #[inline]
    pub fn top_asks(&self, n: usize) -> Vec<Level> {
        self.asks.values().take(n).cloned().collect()
    }

    #[inline]
    pub fn midprice(&self) -> Option<f64> {
        if let (Some(best_bid), Some(best_ask)) = (self.best_bid, self.best_ask) {
            return Some((best_bid.price + best_ask.price) / 2.0);
        }

        None
    }

    #[inline]
    pub fn weighted_midprice(&self) -> Option<f64> {
        if let (Some(best_bid), Some(best_ask)) = (self.best_bid, self.best_ask) {
            let num = best_bid.size * best_ask.price + best_bid.price * best_ask.size;
            let den = best_bid.size + best_ask.size;
            return Some(num / den);
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn process_lvl2_and_trade() {
        let mut ob = Orderbook::new(0.01);

        let event = Event::new(
            "btcusdt".to_string(),
            0,
            0,
            Some(vec![
                Level::new(16.0, 1.0),
                Level::new(17.0, 1.0),
                Level::new(12.0, 1.0),
            ]),
            Some(vec![
                Level::new(19.0, 1.0),
                Level::new(18.0, 1.0),
                Level::new(21.0, 1.0),
            ]),
            None,
            None,
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(3),
            [
                Level::new(17.0, 1.0),
                Level::new(16.0, 1.0),
                Level::new(12.0, 1.0),
            ]
        );

        assert_eq!(
            ob.top_asks(3),
            [
                Level::new(18.0, 1.0),
                Level::new(19.0, 1.0),
                Level::new(21.0, 1.0),
            ]
        );

        // Add more bids and asks
        let event = Event::new(
            "btcusdt".to_string(),
            1,
            1,
            Some(vec![
                Level::new(18.0, 1.0),
                Level::new(11.0, 1.0),
                Level::new(16.5, 1.0),
            ]),
            Some(vec![
                Level::new(17.0, 1.0),
                Level::new(20.0, 1.0),
                Level::new(22.0, 1.0),
            ]),
            None,
            None,
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level::new(18.0, 1.0),
                Level::new(17.0, 1.0),
                Level::new(16.5, 1.0),
                Level::new(16.0, 1.0),
                Level::new(12.0, 1.0),
            ]
        );

        assert_eq!(
            ob.top_asks(5),
            [
                Level::new(17.0, 1.0),
                Level::new(18.0, 1.0),
                Level::new(19.0, 1.0),
                Level::new(20.0, 1.0),
                Level::new(21.0, 1.0),
            ]
        );

        // Update bids and asks
        let event = Event::new(
            "btcusdt".to_string(),
            2,
            2,
            Some(vec![
                Level::new(18.0, 2.0),
                Level::new(17.0, 3.0),
                Level::new(16.5, 4.0),
            ]),
            Some(vec![
                Level::new(17.0, 2.0),
                Level::new(18.0, 3.0),
                Level::new(19.0, 4.0),
            ]),
            None,
            None,
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level::new(18.0, 2.0),
                Level::new(17.0, 3.0),
                Level::new(16.5, 4.0),
                Level::new(16.0, 1.0),
                Level::new(12.0, 1.0),
            ]
        );

        assert_eq!(
            ob.top_asks(5),
            [
                Level::new(17.0, 2.0),
                Level::new(18.0, 3.0),
                Level::new(19.0, 4.0),
                Level::new(20.0, 1.0),
                Level::new(21.0, 1.0),
            ]
        );

        // Remove bid and ask levels
        let event = Event::new(
            "btcusdt".to_string(),
            3,
            3,
            Some(vec![
                Level::new(18.0, 0.0),
                Level::new(17.0, 3.0),
                Level::new(16.5, 0.0),
            ]),
            Some(vec![
                Level::new(17.0, 0.0),
                Level::new(18.0, 3.0),
                Level::new(19.0, 0.0),
            ]),
            None,
            None,
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level::new(17.0, 3.0),
                Level::new(16.0, 1.0),
                Level::new(12.0, 1.0),
                Level::new(11.0, 1.0),
            ]
        );

        assert_eq!(
            ob.top_asks(5),
            [
                Level::new(18.0, 3.0),
                Level::new(20.0, 1.0),
                Level::new(21.0, 1.0),
                Level::new(22.0, 1.0),
            ]
        );

        // Update bid level with trade
        let event = Event::new(
            "btcusdt".to_string(),
            4,
            4,
            None,
            None,
            Some(Level::new(16.0, 0.5)),
            Some(true),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level::new(17.0, 3.0),
                Level::new(16.0, 0.5),
                Level::new(12.0, 1.0),
                Level::new(11.0, 1.0),
            ]
        );

        assert_eq!(
            ob.top_asks(5),
            [
                Level::new(18.0, 3.0),
                Level::new(20.0, 1.0),
                Level::new(21.0, 1.0),
                Level::new(22.0, 1.0),
            ]
        );

        // Update asks level with trade
        let event = Event::new(
            "btcusdt".to_string(),
            5,
            5,
            None,
            None,
            Some(Level::new(20.0, 0.25)),
            Some(false),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level::new(17.0, 3.0),
                Level::new(16.0, 0.5),
                Level::new(12.0, 1.0),
                Level::new(11.0, 1.0),
            ]
        );

        assert_eq!(
            ob.top_asks(5),
            [
                Level::new(18.0, 3.0),
                Level::new(20.0, 0.75),
                Level::new(21.0, 1.0),
                Level::new(22.0, 1.0),
            ]
        );

        // Remove bid price level with trade > current price level quantity
        let event = Event::new(
            "btcusdt".to_string(),
            6,
            6,
            None,
            None,
            Some(Level::new(16.0, 0.75)),
            Some(true),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level::new(17.0, 3.0),
                Level::new(12.0, 1.0),
                Level::new(11.0, 1.0),
            ]
        );

        assert_eq!(
            ob.top_asks(5),
            [
                Level::new(18.0, 3.0),
                Level::new(20.0, 0.75),
                Level::new(21.0, 1.0),
                Level::new(22.0, 1.0),
            ]
        );

        // Remove ask price level with trade > current price level quantity
        let event = Event::new(
            "btcusdt".to_string(),
            7,
            7,
            None,
            None,
            Some(Level::new(20.0, 0.75)),
            Some(false),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [
                Level::new(17.0, 3.0),
                Level::new(12.0, 1.0),
                Level::new(11.0, 1.0),
            ]
        );

        assert_eq!(
            ob.top_asks(5),
            [
                Level::new(18.0, 3.0),
                Level::new(21.0, 1.0),
                Level::new(22.0, 1.0),
            ]
        );

        // Remove bid price level with trade == price level quantity
        let event = Event::new(
            "btcusdt".to_string(),
            8,
            8,
            None,
            None,
            Some(Level::new(17.0, 3.0)),
            Some(true),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
        );

        assert_eq!(
            ob.top_asks(5),
            [
                Level::new(18.0, 3.0),
                Level::new(21.0, 1.0),
                Level::new(22.0, 1.0),
            ]
        );

        // Remove ask price level with trade == price level quantity
        let event = Event::new(
            "btcusdt".to_string(),
            9,
            9,
            None,
            None,
            Some(Level::new(18.0, 3.0)),
            Some(false),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
        );

        assert_eq!(
            ob.top_asks(5),
            [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
        );

        // Remove price level that does not exist with trade
        let event = Event::new(
            "btcusdt".to_string(),
            10,
            10,
            None,
            None,
            Some(Level::new(18.0, 3.0)),
            Some(true),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
        );

        assert_eq!(
            ob.top_asks(5),
            [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
        );

        // Remove ask price level that does not exist
        let event = Event::new(
            "btcusdt".to_string(),
            11,
            11,
            None,
            None,
            Some(Level::new(18.0, 3.0)),
            Some(false),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
        );

        assert_eq!(
            ob.top_asks(5),
            [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
        );

        // Break sequence id by providing seq id < last_sequence
        let event = Event::new(
            "btcusdt".to_string(),
            12,
            10,
            None,
            None,
            Some(Level::new(21.0, 3.0)),
            Some(false),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
        );

        assert_eq!(
            ob.top_asks(5),
            [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
        );

        // Break sequence id by providing timestamp < last_updated
        let event = Event::new(
            "btcusdt".to_string(),
            10,
            12,
            None,
            None,
            Some(Level::new(21.0, 3.0)),
            Some(false),
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(5),
            [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
        );

        assert_eq!(
            ob.top_asks(5),
            [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
        );
    }

    #[test]
    fn process_lvl2_no_asks() {
        let mut ob = Orderbook::new(0.01);

        let event = Event::new(
            "btcusdt".to_string(),
            0,
            0,
            Some(vec![
                Level::new(16.0, 1.0),
                Level::new(17.0, 1.0),
                Level::new(12.0, 1.0),
            ]),
            None,
            None,
            None,
        );

        ob.process(event);

        assert_eq!(
            ob.top_bids(3),
            [
                Level::new(17.0, 1.0),
                Level::new(16.0, 1.0),
                Level::new(12.0, 1.0),
            ]
        );
    }

    #[test]
    fn process_lvl2_no_bids() {
        let mut ob = Orderbook::new(0.01);

        let event = Event::new(
            "btcusdt".to_string(),
            0,
            0,
            None,
            Some(vec![
                Level::new(19.0, 1.0),
                Level::new(18.0, 1.0),
                Level::new(21.0, 1.0),
            ]),
            None,
            None,
        );

        ob.process(event);

        assert_eq!(
            ob.top_asks(3),
            [
                Level::new(18.0, 1.0),
                Level::new(19.0, 1.0),
                Level::new(21.0, 1.0),
            ]
        );
    }

    #[test]
    fn price_tick() {
        let price_tick = price_ticks(120.0, 0.01);
        assert_eq!(price_tick, 1.2 as u64)
    }

    #[test]
    fn midprice() {
        let mut ob = Orderbook::new(0.01);

        let event = Event::new(
            "btcusdt".to_string(),
            0,
            0,
            Some(vec![
                Level::new(16.0, 1.0),
                Level::new(17.0, 1.0),
                Level::new(12.0, 1.0),
            ]),
            Some(vec![
                Level::new(19.0, 1.0),
                Level::new(18.0, 1.0),
                Level::new(21.0, 1.0),
            ]),
            None,
            None,
        );

        ob.process(event);

        let mid_price = ob.midprice().unwrap();

        assert_eq!(mid_price, 17.5);
    }

    #[test]
    fn weighted_midprice() {
        let mut ob = Orderbook::new(0.01);

        let event = Event::new(
            "btcusdt".to_string(),
            0,
            0,
            Some(vec![Level::new(16.0, 1.0)]),
            Some(vec![Level::new(20.0, 4.0)]),
            None,
            None,
        );

        ob.process(event);

        let weighted_midprice = ob.weighted_midprice().unwrap();

        assert_eq!(weighted_midprice, 16.8);
    }

    #[test]
    fn process_stream_bbo() {
        let mut ob = Orderbook::new(0.01);

        let event = Event::new(
            "btcusdt".to_string(),
            0,
            0,
            Some(vec![
                Level::new(16.0, 1.0),
                Level::new(17.0, 1.0),
                Level::new(12.0, 1.0),
            ]),
            Some(vec![
                Level::new(19.0, 1.0),
                Level::new(18.0, 1.0),
                Level::new(21.0, 1.0),
            ]),
            None,
            None,
        );

        let (best_bid, best_ask) = ob.process_stream_bbo(event).unwrap();

        assert_eq!(best_bid.unwrap(), Level::new(17.0, 1.0));

        assert_eq!(best_ask.unwrap(), Level::new(18.0, 1.0));

        let event = Event::new(
            "btcusdt".to_string(),
            0,
            0,
            None,
            Some(vec![Level::new(23.0, 1.0)]),
            None,
            None,
        );

        assert_eq!(ob.process_stream_bbo(event), None);
    }
}
