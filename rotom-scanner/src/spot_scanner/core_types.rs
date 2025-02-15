use chrono::{DateTime, Duration, Utc};
use ordered_float::OrderedFloat;
use rotom_data::{
    assets::level::Level,
    model::{event_trade::EventTrade, network_info::NetworkSpecData},
    shared::subscription_models::{Coin, ExchangeId, Instrument},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, VecDeque},
};

/*----- */
// Maps
/*----- */
#[derive(Debug, Default, PartialEq)]
pub struct InstrumentMarketDataMap(pub HashMap<Instrument, InstrumentMarketData>);

#[derive(Debug, Default, PartialEq)]
pub struct ExchangeMarketDataMap(pub HashMap<ExchangeId, InstrumentMarketDataMap>);

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SpreadHistoryMap(pub HashMap<ExchangeId, SpreadHistory>);

#[derive(Debug, Default)]
pub struct NetworkStatusMap(pub HashMap<(ExchangeId, Coin), NetworkSpecData>);

/*----- */
// VecDequeTime
/*----- */
#[derive(Debug, PartialEq, Clone)]
pub struct VecDequeTime<T> {
    pub data: VecDeque<(DateTime<Utc>, T)>,
    pub window: Duration,
}

impl<T> Default for VecDequeTime<T> {
    fn default() -> Self {
        Self {
            data: VecDeque::with_capacity(1000),
            window: Duration::minutes(10),
        }
    }
}

impl<T> VecDequeTime<T> {
    pub fn new(time: DateTime<Utc>, value: T) -> Self {
        let mut queue = VecDeque::<(DateTime<Utc>, T)>::with_capacity(1000);
        queue.push_back((time, value));

        Self {
            data: queue,
            window: Duration::minutes(10),
        }
    }

    pub fn push(&mut self, current_time: DateTime<Utc>, value: T) {
        self.clean_up(current_time);
        self.data.push_back((current_time, value));
    }

    fn clean_up(&mut self, current_time: DateTime<Utc>) {
        let time_threshold = current_time - self.window;

        // Clear out any data that is more than n mins (currently 10) or greater
        // than the most recent time
        while let Some((date_time, _)) = self.data.front() {
            if *date_time < time_threshold {
                self.data.pop_front();
            } else {
                break;
            }
        }
    }
}

/*----- */
// Scanner market data
/*----- */
#[derive(Debug, PartialEq, Clone)]
pub struct InstrumentMarketData {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub trades: VecDequeTime<EventTrade>,
    pub spreads: RefCell<SpreadHistoryMap>,
    pub trades_ws_is_connected: bool,
    pub orderbook_ws_is_connected: bool,
}

impl Default for InstrumentMarketData {
    fn default() -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
            trades: VecDequeTime::default(),
            spreads: RefCell::new(SpreadHistoryMap(HashMap::with_capacity(10))),
            orderbook_ws_is_connected: false,
            trades_ws_is_connected: false,
        }
    }
}

impl InstrumentMarketData {
    pub fn new_orderbook(bids: Vec<Level>, asks: Vec<Level>) -> Self {
        Self {
            bids,
            asks,
            trades: VecDequeTime::default(),
            spreads: RefCell::new(SpreadHistoryMap(HashMap::with_capacity(10))),
            orderbook_ws_is_connected: true,
            trades_ws_is_connected: false,
        }
    }

    pub fn new_trade(time: DateTime<Utc>, value: EventTrade) -> Self {
        Self {
            bids: Vec::with_capacity(10),
            asks: Vec::with_capacity(10),
            trades: VecDequeTime::new(time, value),
            spreads: RefCell::new(SpreadHistoryMap(HashMap::with_capacity(10))),
            orderbook_ws_is_connected: false,
            trades_ws_is_connected: true,
        }
    }

    pub fn bid_ask_has_no_data(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }
}

/*----- */
// Spread History
/*----- */
#[derive(Debug, Default, PartialEq, Clone)]
pub struct SpreadHistory {
    pub take_take: VecDequeTime<f64>,
    pub take_make: VecDequeTime<f64>,
    pub make_take: VecDequeTime<f64>,
    pub make_make: VecDequeTime<f64>,
}

impl SpreadHistory {
    pub fn new_ask(take_take: f64, take_make: f64) -> Self {
        let mut take_take_queue = VecDequeTime::default();
        let mut take_make_queue = VecDequeTime::default();

        take_take_queue.push(Utc::now(), take_take);
        take_make_queue.push(Utc::now(), take_make);

        Self {
            take_take: take_take_queue,
            take_make: take_make_queue,
            make_take: VecDequeTime::default(),
            make_make: VecDequeTime::default(),
        }
    }

    pub fn new_bid(make_take: f64, make_make: f64) -> Self {
        let mut make_take_queue = VecDequeTime::default();
        let mut make_make_queue = VecDequeTime::default();

        make_take_queue.push(Utc::now(), make_take);
        make_make_queue.push(Utc::now(), make_make);

        Self {
            take_take: VecDequeTime::default(),
            take_make: VecDequeTime::default(),
            make_take: make_take_queue,
            make_make: make_make_queue,
        }
    }

    pub fn insert(&mut self, time: DateTime<Utc>, spread_array: [Option<f64>; 4]) {
        if let Some(take_take) = spread_array[0] {
            self.take_take.push(time, take_take);
        }

        if let Some(take_make) = spread_array[1] {
            self.take_make.push(time, take_make);
        }

        if let Some(make_take) = spread_array[2] {
            self.make_take.push(time, make_take);
        }

        if let Some(make_make) = spread_array[3] {
            self.make_make.push(time, make_make);
        }
    }
}

/*----- */
// Spread Key
/*----- */
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct SpreadKey {
    exchanges: (ExchangeId, ExchangeId),
    instrument: Instrument,
}

impl SpreadKey {
    pub fn new(
        spread_change_exchange: ExchangeId,
        existing_exchange: ExchangeId,
        instrument: Instrument,
    ) -> Self {
        Self {
            exchanges: (spread_change_exchange, existing_exchange),
            instrument,
        }
    }
}

/*----- */
// Spread Sorted
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
