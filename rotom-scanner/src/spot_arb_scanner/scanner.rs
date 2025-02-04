use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Duration, Utc};
use rotom_data::{
    assets::level::Level,
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
    pub buy_illiquid_spreads: VecDequeTime<f64>,
    pub sell_illiquid_spreads: VecDequeTime<f64>,
    pub buy_liquid_spreads: VecDequeTime<f64>,
    pub sell_liquid_spreads: VecDequeTime<f64>,
}

#[derive(Debug)]
pub struct SpotArbMarketDataMap(pub HashMap<(ExchangeId, Instrument), InstrumentMarketData>);
