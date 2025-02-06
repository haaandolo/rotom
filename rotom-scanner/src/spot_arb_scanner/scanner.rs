use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, VecDeque},
};

use chrono::{DateTime, Duration, Utc};
use ordered_float::OrderedFloat;
use rotom_data::{
    assets::level::Level,
    model::network_info::NetworkSpecs,
    shared::subscription_models::{ExchangeId, Instrument},
};

#[derive(Debug, Default)]
pub struct VecDequeTime<T> {
    pub data: VecDeque<(DateTime<Utc>, T)>,
    pub window: Duration,
}

impl<T> VecDequeTime<T> {
    pub fn new() -> Self {
        Self {
            data: VecDeque::with_capacity(1000),
            window: Duration::minutes(10),
        }
    }

    pub fn push(&mut self, current_time: DateTime<Utc>, value: T) {
        self.clean_up(current_time);
        self.data.push_back((current_time, value));
    }

    fn clean_up(&mut self, current_time: DateTime<Utc>) {
        let time_threshold = current_time - self.window;

        while let Some((date_time, _)) = self.data.front() {
            if *date_time < time_threshold {
                self.data.pop_front();
            } else {
                break;
            }
        }
    }
}

#[derive(Debug)]
pub struct InstrumentMarketData {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub average_trade_price: VecDequeTime<f64>,
    pub average_trade_quantity: VecDequeTime<f64>,
}

#[derive(Debug, Default)]
pub struct SpreadHistory {
    pub current_spread: f64,
    pub buy_illiquid_spreads: VecDequeTime<f64>,
    pub sell_illiquid_spreads: VecDequeTime<f64>,
    pub buy_liquid_spreads: VecDequeTime<f64>,
    pub sell_liquid_spreads: VecDequeTime<f64>,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct SpreadKey {
    exchanges: (ExchangeId, ExchangeId),
    instrument: Instrument,
}

impl SpreadKey {
    pub fn new(e1: ExchangeId, e2: ExchangeId, instrument: Instrument) -> Self {
        match e1.cmp(&e2) {
            Ordering::Greater => SpreadKey {
                exchanges: (e2, e1),
                instrument,
            },
            _ => SpreadKey {
                exchanges: (e1, e2),
                instrument,
            },
        }
    }
}

/*----- */
// Maps
/*----- */
#[derive(Debug, Default)]
pub struct InstrumentMarketDataMap(pub HashMap<Instrument, InstrumentMarketData>);

#[derive(Debug, Default)]
pub struct ExchangeMarketDataMap(pub HashMap<ExchangeId, InstrumentMarketDataMap>);

#[derive(Debug, Default)]
pub struct SpreadHistoryMap(pub HashMap<SpreadKey, SpreadHistory>);

#[derive(Debug, Default)]
pub struct NetworkStatusMap(pub HashMap<ExchangeId, NetworkSpecs>);

/*----- */
// Spot Arb Scanner
/*----- */
#[derive(Debug)]
pub struct SpotArbScanner {
    pub exchange_data: ExchangeMarketDataMap,
    pub network_status: NetworkStatusMap,
    pub spread_history: SpreadHistoryMap,
    pub spreads_sorted: SpreadsSorted,
}

/*----- */
// Data structure to hold sorted spread values
/*----- */
#[derive(Debug, Default)]
pub struct SpreadsSorted {
    by_key: BTreeMap<SpreadKey, OrderedFloat<f64>>,
    by_value: BTreeMap<OrderedFloat<f64>, SpreadKey>,
}

impl SpreadsSorted {
    pub fn new() -> Self {
        Self {
            by_key: BTreeMap::new(),
            by_value: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, spread_key: SpreadKey, new_spread: f64) {
        match self.by_key.get_mut(&spread_key) {
            // If key exists and the old_spread != new_spread, then modify the
            // old spread to be new spread in the btreemap (by_key). And remove the
            // old spread in the btreemap (by_value) and insert the new spread
            Some(old_spread) => {
                if old_spread != &new_spread {
                    self.by_value.remove(old_spread);
                    *old_spread = OrderedFloat(new_spread);
                    self.by_value.insert(OrderedFloat(new_spread), spread_key);
                }
            }
            // Else just insert the new spread and spread key in the both btreemaps
            None => {
                self.by_value
                    .insert(OrderedFloat(new_spread), spread_key.clone());
                self.by_key.insert(spread_key, OrderedFloat(new_spread));
            }
        }
    }

    pub fn snapshot(&self) -> Vec<(f64, SpreadKey)> {
        self.by_value
            .iter()
            .rev()
            .take(10)
            .map(|(spread, spread_key)| (spread.0, spread_key.clone()))
            .collect::<Vec<_>>()
    }
}

/*----- */
// Test
/*----- */
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn spread_sorted_test() {
        // Init keys
        let k1 = SpreadKey::new(
            ExchangeId::AscendExSpot,
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
        );

        let k2 = SpreadKey::new(
            ExchangeId::BinanceSpot,
            ExchangeId::AscendExSpot,
            Instrument::new("btc", "usdt"),
        );

        let k3 = SpreadKey::new(
            ExchangeId::BinanceSpot,
            ExchangeId::ExmoSpot,
            Instrument::new("eth", "usdt"),
        );

        let k4 = SpreadKey::new(
            ExchangeId::WooxSpot,
            ExchangeId::ExmoSpot,
            Instrument::new("op", "usdt"),
        );

        // Init spreads
        let s1 = 0.005; // 2
        let s2 = 0.0005; // 3
        let s3 = 0.01; // 1
        let s4 = 0.000025; // 4

        // Init spread map
        let mut spread_map = SpreadsSorted::new();

        // Insert keys with same exchange combo
        spread_map.insert(k1.clone(), s1);
        spread_map.insert(k2.clone(), s2);
        let result = spread_map.snapshot();
        let expected = vec![
            (s2, k2.clone()),
        ];
        assert_eq!(result, expected);

        // Insert other key that have different exchange combo to map
        spread_map.insert(k3.clone(), s3);
        spread_map.insert(k4.clone(), s4);

        let result = spread_map.snapshot();
        let expected = vec![
            (s3, k3.clone()),
            (s2, k2.clone()),
            (s4, k4.clone()),
        ];
        assert_eq!(result, expected);

        // Change exisiting key to be the top value
        let s5 = 0.1;
        spread_map.insert(k4.clone(), s5);
        let result = spread_map.snapshot();
        let expected = vec![
            (s5, k4.clone()),
            (s3, k3.clone()),
            (s2, k2.clone()),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn spread_key_test() {
        let k1 = SpreadKey::new(
            ExchangeId::AscendExSpot,
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
        );

        let k2 = SpreadKey::new(
            ExchangeId::BinanceSpot,
            ExchangeId::AscendExSpot,
            Instrument::new("btc", "usdt"),
        );

        assert_eq!(k1, k2)
    }
}
