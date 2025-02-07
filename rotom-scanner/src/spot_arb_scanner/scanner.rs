use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, VecDeque},
};

use chrono::{DateTime, Duration, Utc};
use ordered_float::OrderedFloat;
use rotom_data::{
    assets::level::Level,
    model::{
        event_trade::EventTrade,
        market_event::{DataKind, MarketEvent},
        network_info::{NetworkSpecData, NetworkSpecs},
    },
    shared::subscription_models::{Coin, ExchangeId, Instrument},
};
use tokio::sync::mpsc;
use tracing::warn;

#[derive(Debug)]
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

        while let Some((date_time, _)) = self.data.front() {
            if *date_time < time_threshold {
                self.data.pop_front();
            } else {
                break;
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct InstrumentMarketData {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub trades: VecDequeTime<EventTrade>,
}

impl InstrumentMarketData {
    pub fn new_orderbook(bids: Vec<Level>, asks: Vec<Level>) -> Self {
        Self {
            bids,
            asks,
            trades: VecDequeTime::default(),
        }
    }

    pub fn new_trade(time: DateTime<Utc>, value: EventTrade) -> Self {
        Self {
            bids: Vec::with_capacity(10),
            asks: Vec::with_capacity(10),
            trades: VecDequeTime::new(time, value),
        }
    }
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
pub struct NetworkStatusMap(pub HashMap<(ExchangeId, Coin), NetworkSpecData>);

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
// Spot Arb Scanner
/*----- */
#[derive(Debug)]
pub struct SpotArbScanner {
    pub exchange_data: ExchangeMarketDataMap,
    pub network_status: NetworkStatusMap,
    pub spread_history: SpreadHistoryMap,
    pub spreads_sorted: SpreadsSorted,
    pub network_status_stream: mpsc::UnboundedReceiver<NetworkSpecs>,
    pub market_data_stream: mpsc::UnboundedReceiver<MarketEvent<DataKind>>,
}

impl SpotArbScanner {
    pub fn new(
        network_status_stream: mpsc::UnboundedReceiver<NetworkSpecs>,
        market_data_stream: mpsc::UnboundedReceiver<MarketEvent<DataKind>>,
    ) -> Self {
        Self {
            exchange_data: ExchangeMarketDataMap::default(),
            network_status: NetworkStatusMap::default(),
            spread_history: SpreadHistoryMap::default(),
            spreads_sorted: SpreadsSorted::default(),
            network_status_stream,
            market_data_stream,
        }
    }

    fn process_orderbook(
        &mut self,
        exchange: ExchangeId,
        instrument: Instrument,
        mut bids: Vec<Level>,
        mut asks: Vec<Level>,
    ) {
        self.exchange_data
            .0
            .entry(exchange)
            .or_default()
            .0
            .entry(instrument)
            .and_modify(|market_data_state| {
                std::mem::swap(&mut market_data_state.bids, &mut bids);
                std::mem::swap(&mut market_data_state.asks, &mut asks);
            })
            .or_insert_with(|| InstrumentMarketData::new_orderbook(bids, asks));
    }

    fn process_trade(
        &mut self,
        exchange: ExchangeId,
        instrument: Instrument,
        time: DateTime<Utc>,
        trade: EventTrade,
    ) {
        self.exchange_data
            .0
            .entry(exchange)
            .or_default()
            .0
            .entry(instrument)
            .and_modify(|market_data_state| {
                market_data_state.trades.push(time, trade.clone());
            })
            .or_insert_with(|| InstrumentMarketData::new_trade(time, trade));
    }

    fn process_trades(
        &mut self,
        exchange: ExchangeId,
        instrument: Instrument,
        time: DateTime<Utc>,
        trades: Vec<EventTrade>,
    ) {
        self.exchange_data
            .0
            .entry(exchange)
            .or_default()
            .0
            .entry(instrument)
            .and_modify(|market_data_state| {
                trades
                    .iter()
                    .for_each(|trade| market_data_state.trades.push(time, trade.to_owned()));
            })
            .or_insert_with(|| {
                let mut instrument_map = InstrumentMarketData::default();
                trades
                    .iter()
                    .for_each(|trade| instrument_map.trades.push(time, trade.to_owned()));
                instrument_map
            });
    }

    fn process_network_status(&mut self, network_status: NetworkSpecs) {
        for (key, value) in network_status.0.into_iter() {
            self.network_status.0.insert(key, value);
        }
    }

    pub fn run(mut self) {
        'spot_arb_scanner: loop {
            // Replace network status state
            match self.network_status_stream.try_recv() {
                Ok(network_status_update) => {
                    self.process_network_status(network_status_update);
                }
                Err(error) => {
                    if error == mpsc::error::TryRecvError::Disconnected {
                        warn!(
                            message = "Network status stream for spot arb scanner has disconnected",
                            action = "Breaking Spot Arb Scanner",
                        );
                        break 'spot_arb_scanner;
                    }
                }
            }

            // Get Market Data update
            match self.market_data_stream.try_recv() {
                Ok(market_data) => {
                    println!("### before update ### \n {:?}", market_data);
                    match market_data.event_data {
                        DataKind::OrderBook(orderbook) => self.process_orderbook(
                            market_data.exchange,
                            market_data.instrument,
                            orderbook.bids,
                            orderbook.asks,
                        ),
                        DataKind::OrderBookSnapshot(snapshot) => self.process_orderbook(
                            market_data.exchange,
                            market_data.instrument,
                            snapshot.bids,
                            snapshot.asks,
                        ),
                        DataKind::Trade(trade) => self.process_trade(
                            market_data.exchange,
                            market_data.instrument,
                            market_data.received_time,
                            trade,
                        ),
                        DataKind::Trades(trades) => self.process_trades(
                            market_data.exchange,
                            market_data.instrument,
                            market_data.received_time,
                            trades,
                        ),
                    }

                    println!("$$$");
                    println!("### after update ### \n {:?}", self.exchange_data.0);
                    println!("###################################################")
                }
                Err(error) => {
                    if error == mpsc::error::TryRecvError::Disconnected {
                        warn!(
                            message = "Network status stream for spot arb scanner has disconnected",
                            action = "Breaking Spot Arb Scanner",
                        );
                        break 'spot_arb_scanner;
                    }
                }
            };
        }
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
        let expected = vec![(s2, k2.clone())];
        assert_eq!(result, expected);

        // Insert other key that have different exchange combo to map
        spread_map.insert(k3.clone(), s3);
        spread_map.insert(k4.clone(), s4);

        let result = spread_map.snapshot();
        let expected = vec![(s3, k3.clone()), (s2, k2.clone()), (s4, k4.clone())];
        assert_eq!(result, expected);

        // Change exisiting key to be the top value
        let s5 = 0.1;
        spread_map.insert(k4.clone(), s5);
        let result = spread_map.snapshot();
        let expected = vec![(s5, k4.clone()), (s3, k3.clone()), (s2, k2.clone())];
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
