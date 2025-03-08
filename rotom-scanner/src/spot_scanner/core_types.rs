use chrono::{DateTime, Duration, Utc};
use ordered_float::OrderedFloat;
use rotom_data::{
    assets::level::Level,
    model::{event_trade::EventTrade, network_info::NetworkSpecData},
    shared::subscription_models::{Coin, ExchangeId, Instrument},
};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, VecDeque};

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
#[derive(Debug, PartialEq, Clone, Serialize)]
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
#[derive(Debug, Serialize, Default)]
pub struct AverageTradeInfo {
    pub avg_price: f64, // avg price in last 10min
    pub avg_notional: f64,
    pub avg_buy_notional: f64,
    pub avg_sell_notional: f64,
    pub buy_sell_ratio: f64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InstrumentMarketData {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub trades: VecDequeTime<EventTrade>,
    pub spreads: SpreadHistoryMap,
    pub trades_ws_is_connected: bool,
    pub orderbook_ws_is_connected: bool,
    pub trades_last_update_time: DateTime<Utc>,
    pub orderbook_last_update_time: DateTime<Utc>,
}

impl InstrumentMarketData {
    pub fn new(update_time: DateTime<Utc>) -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
            trades: VecDequeTime::default(),
            spreads: SpreadHistoryMap(HashMap::with_capacity(10)),
            orderbook_ws_is_connected: false,
            trades_ws_is_connected: false,
            trades_last_update_time: update_time,
            orderbook_last_update_time: update_time,
        }
    }

    pub fn new_orderbook(update_time: DateTime<Utc>, bids: Vec<Level>, asks: Vec<Level>) -> Self {
        Self {
            bids,
            asks,
            trades: VecDequeTime::default(),
            spreads: SpreadHistoryMap(HashMap::with_capacity(10)),
            orderbook_ws_is_connected: true,
            trades_ws_is_connected: false,
            trades_last_update_time: update_time,
            orderbook_last_update_time: update_time,
        }
    }

    pub fn new_trade(update_time: DateTime<Utc>, value: EventTrade) -> Self {
        Self {
            bids: Vec::with_capacity(10),
            asks: Vec::with_capacity(10),
            trades: VecDequeTime::new(update_time, value),
            spreads: SpreadHistoryMap(HashMap::with_capacity(10)),
            orderbook_ws_is_connected: false,
            trades_ws_is_connected: true,
            trades_last_update_time: update_time,
            orderbook_last_update_time: update_time,
        }
    }

    pub fn get_average_trades(&self) -> AverageTradeInfo {
        if self.trades.data.is_empty() {
            return AverageTradeInfo::default();
        }

        let (price_sum, total_volume, buy_count, count, buy_volume, sell_volume) =
            self.trades.data.iter().fold(
                (0.0, 0.0, 0, 0.0, 0.0, 0.0),
                |(price_sum, size_sum, buy_count, count, buy_volume, sell_volume), (_, trade)| {
                    let new_buy_volume = if trade.is_buy {
                        buy_volume + trade.trade.size
                    } else {
                        buy_volume
                    };

                    let new_sell_volume = if !trade.is_buy {
                        sell_volume + trade.trade.size
                    } else {
                        sell_volume
                    };

                    (
                        price_sum + trade.trade.price,
                        size_sum + trade.trade.size,
                        buy_count + trade.is_buy as usize,
                        count + 1.0,
                        new_buy_volume,
                        new_sell_volume,
                    )
                },
            );

        let avg_price = price_sum / count;

        AverageTradeInfo {
            avg_price,
            avg_notional: avg_price * total_volume,
            avg_buy_notional: avg_price * buy_volume,
            avg_sell_notional: avg_price * sell_volume,
            buy_sell_ratio: buy_count as f64 / count,
        }
    }
}

/*----- */
// Spread History
/*----- */
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize)]
pub struct LatestSpreads {
    pub take_take: f64,
    pub take_make: f64,
    pub make_take: f64,
}

#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct SpreadHistory {
    pub latest_spreads: LatestSpreads,
    pub take_take: VecDequeTime<f64>,
    pub take_make: VecDequeTime<f64>,
    pub make_take: VecDequeTime<f64>,
}

impl SpreadHistory {
    pub fn insert(&mut self, time: DateTime<Utc>, spread_array: [Option<f64>; 3]) {
        if let Some(take_take) = spread_array[0] {
            self.latest_spreads.take_take = take_take;
            self.take_take.push(time, take_take);
        }

        if let Some(take_make) = spread_array[1] {
            self.latest_spreads.take_make = take_make;
            self.take_make.push(time, take_make);
        }

        if let Some(make_take) = spread_array[2] {
            self.latest_spreads.make_take = make_take;
            self.make_take.push(time, make_take);
        }
    }
}

/*----- */
// Spread Key
/*----- */
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct SpreadKey {
    pub exchanges: (ExchangeId, ExchangeId),
    pub instrument: Instrument,
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
            .take(150)
            .map(|(spread, spread_key)| (spread.0, spread_key.clone()))
            .collect::<Vec<_>>()
    }
}
