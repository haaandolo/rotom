use chrono::{DateTime, Utc};

use crate::data::model::event_book::EventOrderBook;
use crate::data::model::level::Level;
use std::collections::BTreeMap;

/*----- */
// Orderbook
/*----- */
#[derive(Debug, Default, Clone)]
pub struct OrderBook {
    best_bid: Option<Level>,
    best_ask: Option<Level>,
    bids: BTreeMap<u64, Level>,
    asks: BTreeMap<u64, Level>,
    inv_tick_size: f64,
    pub last_update_time: DateTime<Utc>,
}

impl OrderBook {
    pub fn new(tick_size: f64) -> Self {
        Self {
            best_bid: None,
            best_ask: None,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            inv_tick_size: 1.0 / tick_size,
            last_update_time: Utc::now(),
        }
    }

    #[inline]
    fn price_ticks(&self, price: f64, tick_size: f64) -> u64 {
        (price * tick_size) as u64
    }

    #[inline]
    pub fn process_lvl2(&mut self, bids: Option<Vec<Level>>, asks: Option<Vec<Level>>) {
        // Process bids
        if let Some(levels) = bids {
            levels.into_iter().for_each(|level| {
                let price_tick = self.price_ticks(level.price, self.inv_tick_size);
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
        if let Some(levels) = asks {
            levels.into_iter().for_each(|level| {
                let price_tick = self.price_ticks(level.price, self.inv_tick_size);
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
    }

    #[inline]
    pub fn snapshot(&self) -> Self {
        self.clone()
    }

    #[inline]
    pub fn book_snapshot(&self) -> EventOrderBook {
        let bids = self.bids.values().rev().take(10).cloned().collect();
        let asks = self.asks.values().take(10).cloned().collect();
        EventOrderBook::new(bids, asks)
    }

    #[inline]
    pub fn reset(&mut self) {
        self.best_bid = None;
        self.best_ask = None;
        self.bids = BTreeMap::new();
        self.asks = BTreeMap::new();
    }
    /*----- Good to have functions ----- */
    // #[inline]
    // fn process_trade(&mut self, event: Event) {
    //     let buf = if event.is_buy == Some(true) {
    //         &mut self.bids
    //     } else {
    //         &mut self.asks
    //     };

    //     let trade_level = event.trade.unwrap();
    //     let price_ticks = self.price_ticks(trade_level.price, self.inv_tick_size);

    //     if let Some(level) = buf.get_mut(&price_ticks) {
    //         if trade_level.size >= level.size {
    //             buf.remove(&price_ticks);
    //         } else {
    //             level.size -= trade_level.size;
    //         }
    //     };
    // }

    // pub fn best_bid(&self) -> Option<Level> {
    //     self.best_bid
    // }

    // pub fn best_ask(&self) -> Option<Level> {
    //     self.best_ask
    // }

    // #[inline]
    // pub fn top_bids(&self, n: usize) -> Vec<Level> {
    //     self.bids.values().rev().take(n).cloned().collect()
    // }

    // #[inline]
    // pub fn top_asks(&self, n: usize) -> Vec<Level> {
    //     self.asks.values().take(n).cloned().collect()
    // }

    // #[inline]
    // pub fn midprice(&self) -> Option<f64> {
    //     if let (Some(best_bid), Some(best_ask)) = (self.best_bid, self.best_ask) {
    //         return Some((best_bid.price + best_ask.price) / 2.0);
    //     }

    //     None
    // }

    // #[inline]
    // pub fn weighted_midprice(&self) -> Option<f64> {
    //     if let (Some(best_bid), Some(best_ask)) = (self.best_bid, self.best_ask) {
    //         let num = best_bid.size * best_ask.price + best_bid.price * best_ask.size;
    //         let den = best_bid.size + best_ask.size;
    //         return Some(num / den);
    //     }

    //     None
    // }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn process_lvl2_and_trade() {
//         let mut ob = Orderbook::new(0.01);

//         let event = Event::new(
//             "btcusdt".to_string(),
//             0,
//             0,
//             Some(vec![
//                 Level::new(16.0, 1.0),
//                 Level::new(17.0, 1.0),
//                 Level::new(12.0, 1.0),
//             ]),
//             Some(vec![
//                 Level::new(19.0, 1.0),
//                 Level::new(18.0, 1.0),
//                 Level::new(21.0, 1.0),
//             ]),
//             None,
//             None,
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(3),
//             [
//                 Level::new(17.0, 1.0),
//                 Level::new(16.0, 1.0),
//                 Level::new(12.0, 1.0),
//             ]
//         );

//         assert_eq!(
//             ob.top_asks(3),
//             [
//                 Level::new(18.0, 1.0),
//                 Level::new(19.0, 1.0),
//                 Level::new(21.0, 1.0),
//             ]
//         );

//         // Add more bids and asks
//         let event = Event::new(
//             "btcusdt".to_string(),
//             1,
//             1,
//             Some(vec![
//                 Level::new(18.0, 1.0),
//                 Level::new(11.0, 1.0),
//                 Level::new(16.5, 1.0),
//             ]),
//             Some(vec![
//                 Level::new(17.0, 1.0),
//                 Level::new(20.0, 1.0),
//                 Level::new(22.0, 1.0),
//             ]),
//             None,
//             None,
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [
//                 Level::new(18.0, 1.0),
//                 Level::new(17.0, 1.0),
//                 Level::new(16.5, 1.0),
//                 Level::new(16.0, 1.0),
//                 Level::new(12.0, 1.0),
//             ]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [
//                 Level::new(17.0, 1.0),
//                 Level::new(18.0, 1.0),
//                 Level::new(19.0, 1.0),
//                 Level::new(20.0, 1.0),
//                 Level::new(21.0, 1.0),
//             ]
//         );

//         // Update bids and asks
//         let event = Event::new(
//             "btcusdt".to_string(),
//             2,
//             2,
//             Some(vec![
//                 Level::new(18.0, 2.0),
//                 Level::new(17.0, 3.0),
//                 Level::new(16.5, 4.0),
//             ]),
//             Some(vec![
//                 Level::new(17.0, 2.0),
//                 Level::new(18.0, 3.0),
//                 Level::new(19.0, 4.0),
//             ]),
//             None,
//             None,
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [
//                 Level::new(18.0, 2.0),
//                 Level::new(17.0, 3.0),
//                 Level::new(16.5, 4.0),
//                 Level::new(16.0, 1.0),
//                 Level::new(12.0, 1.0),
//             ]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [
//                 Level::new(17.0, 2.0),
//                 Level::new(18.0, 3.0),
//                 Level::new(19.0, 4.0),
//                 Level::new(20.0, 1.0),
//                 Level::new(21.0, 1.0),
//             ]
//         );

//         // Remove bid and ask levels
//         let event = Event::new(
//             "btcusdt".to_string(),
//             3,
//             3,
//             Some(vec![
//                 Level::new(18.0, 0.0),
//                 Level::new(17.0, 3.0),
//                 Level::new(16.5, 0.0),
//             ]),
//             Some(vec![
//                 Level::new(17.0, 0.0),
//                 Level::new(18.0, 3.0),
//                 Level::new(19.0, 0.0),
//             ]),
//             None,
//             None,
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [
//                 Level::new(17.0, 3.0),
//                 Level::new(16.0, 1.0),
//                 Level::new(12.0, 1.0),
//                 Level::new(11.0, 1.0),
//             ]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [
//                 Level::new(18.0, 3.0),
//                 Level::new(20.0, 1.0),
//                 Level::new(21.0, 1.0),
//                 Level::new(22.0, 1.0),
//             ]
//         );

//         // Update bid level with trade
//         let event = Event::new(
//             "btcusdt".to_string(),
//             4,
//             4,
//             None,
//             None,
//             Some(Level::new(16.0, 0.5)),
//             Some(true),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [
//                 Level::new(17.0, 3.0),
//                 Level::new(16.0, 0.5),
//                 Level::new(12.0, 1.0),
//                 Level::new(11.0, 1.0),
//             ]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [
//                 Level::new(18.0, 3.0),
//                 Level::new(20.0, 1.0),
//                 Level::new(21.0, 1.0),
//                 Level::new(22.0, 1.0),
//             ]
//         );

//         // Update asks level with trade
//         let event = Event::new(
//             "btcusdt".to_string(),
//             5,
//             5,
//             None,
//             None,
//             Some(Level::new(20.0, 0.25)),
//             Some(false),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [
//                 Level::new(17.0, 3.0),
//                 Level::new(16.0, 0.5),
//                 Level::new(12.0, 1.0),
//                 Level::new(11.0, 1.0),
//             ]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [
//                 Level::new(18.0, 3.0),
//                 Level::new(20.0, 0.75),
//                 Level::new(21.0, 1.0),
//                 Level::new(22.0, 1.0),
//             ]
//         );

//         // Remove bid price level with trade > current price level quantity
//         let event = Event::new(
//             "btcusdt".to_string(),
//             6,
//             6,
//             None,
//             None,
//             Some(Level::new(16.0, 0.75)),
//             Some(true),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [
//                 Level::new(17.0, 3.0),
//                 Level::new(12.0, 1.0),
//                 Level::new(11.0, 1.0),
//             ]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [
//                 Level::new(18.0, 3.0),
//                 Level::new(20.0, 0.75),
//                 Level::new(21.0, 1.0),
//                 Level::new(22.0, 1.0),
//             ]
//         );

//         // Remove ask price level with trade > current price level quantity
//         let event = Event::new(
//             "btcusdt".to_string(),
//             7,
//             7,
//             None,
//             None,
//             Some(Level::new(20.0, 0.75)),
//             Some(false),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [
//                 Level::new(17.0, 3.0),
//                 Level::new(12.0, 1.0),
//                 Level::new(11.0, 1.0),
//             ]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [
//                 Level::new(18.0, 3.0),
//                 Level::new(21.0, 1.0),
//                 Level::new(22.0, 1.0),
//             ]
//         );

//         // Remove bid price level with trade == price level quantity
//         let event = Event::new(
//             "btcusdt".to_string(),
//             8,
//             8,
//             None,
//             None,
//             Some(Level::new(17.0, 3.0)),
//             Some(true),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [
//                 Level::new(18.0, 3.0),
//                 Level::new(21.0, 1.0),
//                 Level::new(22.0, 1.0),
//             ]
//         );

//         // Remove ask price level with trade == price level quantity
//         let event = Event::new(
//             "btcusdt".to_string(),
//             9,
//             9,
//             None,
//             None,
//             Some(Level::new(18.0, 3.0)),
//             Some(false),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
//         );

//         // Remove price level that does not exist with trade
//         let event = Event::new(
//             "btcusdt".to_string(),
//             10,
//             10,
//             None,
//             None,
//             Some(Level::new(18.0, 3.0)),
//             Some(true),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
//         );

//         // Remove ask price level that does not exist
//         let event = Event::new(
//             "btcusdt".to_string(),
//             11,
//             11,
//             None,
//             None,
//             Some(Level::new(18.0, 3.0)),
//             Some(false),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
//         );

//         // Break sequence id by providing seq id < last_sequence
//         let event = Event::new(
//             "btcusdt".to_string(),
//             12,
//             10,
//             None,
//             None,
//             Some(Level::new(21.0, 3.0)),
//             Some(false),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
//         );

//         // Break sequence id by providing timestamp < last_updated
//         let event = Event::new(
//             "btcusdt".to_string(),
//             10,
//             12,
//             None,
//             None,
//             Some(Level::new(21.0, 3.0)),
//             Some(false),
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(5),
//             [Level::new(12.0, 1.0), Level::new(11.0, 1.0),]
//         );

//         assert_eq!(
//             ob.top_asks(5),
//             [Level::new(21.0, 1.0), Level::new(22.0, 1.0),]
//         );
//     }

//     #[test]
//     fn process_lvl2_no_asks() {
//         let mut ob = Orderbook::new(0.01);

//         let event = Event::new(
//             "btcusdt".to_string(),
//             0,
//             0,
//             Some(vec![
//                 Level::new(16.0, 1.0),
//                 Level::new(17.0, 1.0),
//                 Level::new(12.0, 1.0),
//             ]),
//             None,
//             None,
//             None,
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_bids(3),
//             [
//                 Level::new(17.0, 1.0),
//                 Level::new(16.0, 1.0),
//                 Level::new(12.0, 1.0),
//             ]
//         );
//     }

//     #[test]
//     fn process_lvl2_no_bids() {
//         let mut ob = Orderbook::new(0.01);

//         let event = Event::new(
//             "btcusdt".to_string(),
//             0,
//             0,
//             None,
//             Some(vec![
//                 Level::new(19.0, 1.0),
//                 Level::new(18.0, 1.0),
//                 Level::new(21.0, 1.0),
//             ]),
//             None,
//             None,
//         );

//         ob.process(event);

//         assert_eq!(
//             ob.top_asks(3),
//             [
//                 Level::new(18.0, 1.0),
//                 Level::new(19.0, 1.0),
//                 Level::new(21.0, 1.0),
//             ]
//         );
//     }

//     #[test]
//     fn price_tick() {
//         let price_tick = price_ticks(120.0, 0.01);
//         assert_eq!(price_tick, 1.2 as u64)
//     }

//     #[test]
//     fn midprice() {
//         let mut ob = Orderbook::new(0.01);

//         let event = Event::new(
//             "btcusdt".to_string(),
//             0,
//             0,
//             Some(vec![
//                 Level::new(16.0, 1.0),
//                 Level::new(17.0, 1.0),
//                 Level::new(12.0, 1.0),
//             ]),
//             Some(vec![
//                 Level::new(19.0, 1.0),
//                 Level::new(18.0, 1.0),
//                 Level::new(21.0, 1.0),
//             ]),
//             None,
//             None,
//         );

//         ob.process(event);

//         let mid_price = ob.midprice().unwrap();

//         assert_eq!(mid_price, 17.5);
//     }

//     #[test]
//     fn weighted_midprice() {
//         let mut ob = Orderbook::new(0.01);

//         let event = Event::new(
//             "btcusdt".to_string(),
//             0,
//             0,
//             Some(vec![Level::new(16.0, 1.0)]),
//             Some(vec![Level::new(20.0, 4.0)]),
//             None,
//             None,
//         );

//         ob.process(event);

//         let weighted_midprice = ob.weighted_midprice().unwrap();

//         assert_eq!(weighted_midprice, 16.8);
//     }

//     #[test]
//     fn process_stream_bbo() {
//         let mut ob = Orderbook::new(0.01);

//         let event = Event::new(
//             "btcusdt".to_string(),
//             0,
//             0,
//             Some(vec![
//                 Level::new(16.0, 1.0),
//                 Level::new(17.0, 1.0),
//                 Level::new(12.0, 1.0),
//             ]),
//             Some(vec![
//                 Level::new(19.0, 1.0),
//                 Level::new(18.0, 1.0),
//                 Level::new(21.0, 1.0),
//             ]),
//             None,
//             None,
//         );

//         let (best_bid, best_ask) = ob.process_stream_bbo(event).unwrap();

//         assert_eq!(best_bid.unwrap(), Level::new(17.0, 1.0));

//         assert_eq!(best_ask.unwrap(), Level::new(18.0, 1.0));

//         let event = Event::new(
//             "btcusdt".to_string(),
//             0,
//             0,
//             None,
//             Some(vec![Level::new(23.0, 1.0)]),
//             None,
//             None,
//         );

//         assert_eq!(ob.process_stream_bbo(event), None);
//     }
// }
