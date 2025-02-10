use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, VecDeque},
};

use chrono::{DateTime, Duration, Utc};
use ordered_float::OrderedFloat;
use rotom_data::{
    assets::level::Level,
    model::{
        event_trade::EventTrade,
        market_event::{DataKind, MarketEvent, WsStatus},
        network_info::{NetworkSpecData, NetworkSpecs},
        EventKind,
    },
    shared::subscription_models::{Coin, ExchangeId, Instrument},
};
use tokio::sync::mpsc;
use tracing::warn;

/*----- */
// VecDeque - Time based
/*----- */
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
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
// Maps - for convenience
/*----- */
#[derive(Debug, Default, PartialEq)]
pub struct InstrumentMarketDataMap(pub HashMap<Instrument, InstrumentMarketData>);

#[derive(Debug, Default, PartialEq)]
pub struct ExchangeMarketDataMap(pub HashMap<ExchangeId, InstrumentMarketDataMap>);

#[derive(Debug, Default, PartialEq)]
pub struct SpreadHistoryMap(pub HashMap<ExchangeId, SpreadHistory>);

#[derive(Debug, Default)]
pub struct NetworkStatusMap(pub HashMap<(ExchangeId, Coin), NetworkSpecData>);

/*----- */
// Spread History
/*----- */
#[derive(Debug, Default, PartialEq)]
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
    pub fn new(e1: ExchangeId, e2: ExchangeId, instrument: Instrument) -> Self {
        Self {
            exchanges: (e1, e2),
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

/*----- */
// Spread Change
/*----- */
#[derive(Debug)]
pub struct SpreadChange {
    pub exchange: ExchangeId,
    pub instrument: Instrument,
    pub bid: Option<Level>,
    pub ask: Option<Level>,
}

impl SpreadChange {
    pub fn new_bid(exchange: ExchangeId, instrument: Instrument, bid: Level) -> Self {
        Self {
            exchange,
            instrument,
            bid: Some(bid),
            ask: None,
        }
    }

    pub fn new_ask(exchange: ExchangeId, instrument: Instrument, ask: Level) -> Self {
        Self {
            exchange,
            instrument,
            bid: None,
            ask: Some(ask),
        }
    }

    pub fn add_bid(&mut self, bid: Level) {
        self.bid = Some(bid);
    }

    pub fn add_ask(&mut self, ask: Level) {
        self.ask = Some(ask);
    }
}

/*----- */
// Spot Arb Scanner
/*----- */
#[derive(Debug)]
pub struct SpotArbScanner {
    exchange_data: ExchangeMarketDataMap,
    network_status: NetworkStatusMap,
    spreads_sorted: SpreadsSorted,
    spread_change_queue: VecDeque<SpreadChange>,
    network_status_stream: mpsc::UnboundedReceiver<NetworkSpecs>,
    market_data_stream: mpsc::UnboundedReceiver<MarketEvent<DataKind>>,
}

impl SpotArbScanner {
    pub fn new(
        network_status_stream: mpsc::UnboundedReceiver<NetworkSpecs>,
        market_data_stream: mpsc::UnboundedReceiver<MarketEvent<DataKind>>,
    ) -> Self {
        Self {
            exchange_data: ExchangeMarketDataMap::default(),
            network_status: NetworkStatusMap::default(),
            spreads_sorted: SpreadsSorted::default(),
            spread_change_queue: VecDeque::with_capacity(10),
            network_status_stream,
            market_data_stream,
        }
    }

    fn did_bba_change(
        exchange: ExchangeId,
        instrument: Instrument,
        old_bid: Level,
        old_ask: Level,
        new_bid: Level,
        new_ask: Level,
    ) -> Option<SpreadChange> {
        let mut result = None;

        if old_bid.price != new_bid.price {
            result = Some(SpreadChange::new_bid(exchange, instrument.clone(), new_bid))
        }

        if old_ask.price != new_ask.price {
            if let Some(ref mut spread_change) = result {
                spread_change.add_ask(new_ask);
            } else {
                result = Some(SpreadChange::new_ask(exchange, instrument, new_ask))
            }
        }

        result
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
            .entry(instrument.clone())
            .and_modify(|market_data_state| {
                // Bid and ask data can be empty if trade data comes in before book data
                // as the InstrumentMarketData::new_trades() sets the bid and ask fields
                // as empty vecs. Hence, we need logic to handle this.

                // Swap with old data if bids is not a empty vec
                if !bids.is_empty() {
                    std::mem::swap(&mut market_data_state.bids, &mut bids);
                }

                // Swap with old data if asks is not a empty vec
                if !asks.is_empty() {
                    std::mem::swap(&mut market_data_state.asks, &mut asks);
                }

                // If we swapped above and the bids and asks are not empty i.e., we
                // haven't just initialised the InstrumentMarketData, then do calculate
                // The spread else do nothing
                if !bids.is_empty() && !asks.is_empty() {
                    let new_bba = Self::did_bba_change(
                        exchange,
                        instrument,
                        bids[0], // We did a mem::swap above so reverse order
                        asks[0], // We did a mem::swap above so reverse order
                        market_data_state.bids[0], // We did a mem::swap above so reverse order
                        market_data_state.asks[0], // We did a mem::swap above so reverse order
                    );

                    // Add to event queue if spread did change
                    if let Some(new_bba) = new_bba {
                        self.spread_change_queue.push_back(new_bba);
                    }
                }
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
        for (key, mut network_spec_data) in network_status.0.into_iter() {
            self.network_status
                .0
                .entry(key)
                .and_modify(|network_status| std::mem::swap(network_status, &mut network_spec_data))
                .or_insert(network_spec_data);
        }
    }

    fn process_spread_change(&mut self, spread_change: SpreadChange) {
        for (exchange, market_data_map) in &self.exchange_data.0 {
            if *exchange != spread_change.exchange {
                if let Some(market_data) = market_data_map.0.get(&spread_change.instrument) {
                    // We assume the exchange associated with the spread change is the buy exchange, so this is in the denominator.
                    // Hence, the sell exchange is the exchange correspoding to the market_data
                    let mut take_take = None;
                    let mut take_make = None;
                    let mut make_take = None;
                    let mut make_make = None;
                    let mut spread_array = [0.0; 4];

                    if let Some(spread_change_ask) = spread_change.ask {
                        // Calculate the spreads if best ask level has changed
                        if !market_data.bids.is_empty() {
                            let take_take_sub =
                                (market_data.bids[0].price / spread_change_ask.price) - 1.0;

                            if take_take_sub > 0.0 {
                                spread_array[0] = take_take_sub;
                            }

                            take_take = Some(take_take_sub)
                        }

                        if !market_data.asks.is_empty() {
                            let take_make_sub =
                                (market_data.asks[0].price / spread_change_ask.price) - 1.0;

                            if take_make_sub > 0.0 {
                                spread_array[1] = take_make_sub;
                            }

                            take_make = Some(take_make_sub)
                        }
                    }

                    if let Some(spread_change_bid) = spread_change.bid {
                        // Calculate the spreads if best bid level has changed
                        if !market_data.bids.is_empty() {
                            let make_take_sub =
                                (market_data.bids[0].price / spread_change_bid.price) - 1.0;

                            if make_take_sub > 0.0 {
                                spread_array[2] = make_take_sub;
                            }

                            make_take = Some(make_take_sub)
                        }

                        if !market_data.asks.is_empty() {
                            let make_make_sub =
                                (market_data.asks[0].price / spread_change_bid.price) - 1.0;

                            if make_make_sub > 0.0 {
                                spread_array[3] = make_make_sub;
                            }

                            make_make = Some(make_make_sub)
                        }
                    }

                    // Update spread history
                    market_data
                        .spreads
                        .borrow_mut()
                        .0
                        .entry(spread_change.exchange)
                        .and_modify(|spread_history| {
                            if let Some(take_take) = take_take {
                                spread_history.take_take.push(Utc::now(), take_take);
                            }

                            if let Some(take_make) = take_make {
                                spread_history.take_make.push(Utc::now(), take_make);
                            }

                            if let Some(make_take) = make_take {
                                spread_history.make_take.push(Utc::now(), make_take);
                            }

                            if let Some(make_make) = make_make {
                                spread_history.make_make.push(Utc::now(), make_make);
                            }
                        })
                        .or_insert(SpreadHistory::default());

                    // Get max spread
                    let max_spread = spread_array[0]
                        .max(spread_array[1])
                        .max(spread_array[2])
                        .max(spread_array[3]);

                    // Insert into spreads_sorted
                    self.spreads_sorted.insert(
                        SpreadKey::new(
                            *exchange,
                            spread_change.exchange,
                            spread_change.instrument.clone(),
                        ),
                        max_spread,
                    );
                }
            }
        }
    }

    fn process_ws_status(
        &mut self,
        exchange: ExchangeId,
        instrument: Instrument,
        ws_status: WsStatus,
    ) {
        self.exchange_data
            .0
            .entry(exchange)
            .or_default()
            .0
            .entry(instrument)
            .and_modify(|market_data_state| match ws_status.get_event_kind() {
                EventKind::OrderBook => {
                    market_data_state.orderbook_ws_is_connected = ws_status.is_connected()
                }
                EventKind::Trade => {
                    market_data_state.trades_ws_is_connected = ws_status.is_connected()
                }
            })
            .or_insert_with(|| match ws_status.get_event_kind() {
                EventKind::OrderBook => InstrumentMarketData {
                    orderbook_ws_is_connected: ws_status.is_connected(),
                    ..InstrumentMarketData::default()
                },
                EventKind::Trade => InstrumentMarketData {
                    trades_ws_is_connected: ws_status.is_connected(),
                    ..InstrumentMarketData::default()
                },
            });
    }

    pub fn run(mut self) {
        'spot_arb_scanner: loop {
            // Process network status update
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

            // Process market data update
            match self.market_data_stream.try_recv() {
                Ok(market_data) => {
                    println!(
                        "###### before update ##### \n {:?} \n ###########",
                        market_data
                    );

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
                        DataKind::ConnectionStatus(ws_status) => self.process_ws_status(
                            market_data.exchange,
                            market_data.instrument,
                            ws_status,
                        ),
                    }

                    // println!(
                    //     "##### after update ##### \n {:?} \n ###########",
                    //     self.exchange_data
                    // );
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

            // Process spreads
            while let Some(spread_change) = self.spread_change_queue.pop_front() {
                // println!("###################");
                // println!("{:?}", spread_change);
                self.process_spread_change(spread_change);
                // println!("{:#?}", self.spreads_sorted.by_value);
            }

            // println!("###################");
            // println!("{:?}",self.exchange_data);
        }
    }
}

/*----- */
// Test
/*----- */
#[cfg(test)]
mod test {
    use chrono::TimeZone;

    use crate::mock_data::test_utils;

    use super::*;

    #[test]
    fn test_scanner_spread() {
        // Init
        let mut scanner = test_utils::spot_arb_scanner();

        // Init Binance market data state
        let binance_btc_ob = test_utils::market_event_orderbook2(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            vec![Level::new(12.0, 10.0)],
            vec![Level::new(13.0, 11.0)],
        );

        scanner.process_orderbook(
            binance_btc_ob.exchange,
            binance_btc_ob.instrument,
            binance_btc_ob.event_data.get_orderbook().unwrap().bids,
            binance_btc_ob.event_data.get_orderbook().unwrap().asks,
        );

        let binance_eth_ob = test_utils::market_event_orderbook2(
            ExchangeId::BinanceSpot,
            Instrument::new("eth", "usdt"),
            vec![Level::new(20.0, 3.0)],
            vec![Level::new(21.0, 12.0)],
        );

        scanner.process_orderbook(
            binance_eth_ob.exchange,
            binance_eth_ob.instrument,
            binance_eth_ob.event_data.get_orderbook().unwrap().bids,
            binance_eth_ob.event_data.get_orderbook().unwrap().asks,
        );

        // Init Binance market data state
        let htx_btc_ob = test_utils::market_event_orderbook2(
            ExchangeId::HtxSpot,
            Instrument::new("btc", "usdt"),
            vec![Level::new(13.5, 11.0)],
            vec![Level::new(14.5, 13.0)],
        );

        scanner.process_orderbook(
            htx_btc_ob.exchange,
            htx_btc_ob.instrument,
            htx_btc_ob.event_data.get_orderbook().unwrap().bids,
            htx_btc_ob.event_data.get_orderbook().unwrap().asks,
        );

        let htx_eth_ob = test_utils::market_event_orderbook2(
            ExchangeId::HtxSpot,
            Instrument::new("eth", "usdt"),
            vec![Level::new(19.22, 5.0)],
            vec![Level::new(20.11, 17.0)],
        );

        scanner.process_orderbook(
            htx_eth_ob.exchange,
            htx_eth_ob.instrument,
            htx_eth_ob.event_data.get_orderbook().unwrap().bids,
            htx_eth_ob.event_data.get_orderbook().unwrap().asks,
        );

        // Binance spread change
        let binance_btc_spread_change = test_utils::spread_change(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            Some(Level::new(12.22, 9.0)),
            Some(Level::new(15.12, 18.03)),
        );

        let binance_eth_spread_change = test_utils::spread_change(
            ExchangeId::BinanceSpot,
            Instrument::new("eth", "usdt"),
            Some(Level::new(20.78, 9.255)),
            Some(Level::new(21.92, 18.78)),
        );

        // Htx spread change
        let htx_btc_spread_change = test_utils::spread_change(
            ExchangeId::HtxSpot,
            Instrument::new("btc", "usdt"),
            Some(Level::new(13.12, 3.0)),
            Some(Level::new(18.02, 19.03)),
        );

        let htx_eth_spread_change = test_utils::spread_change(
            ExchangeId::HtxSpot,
            Instrument::new("eth", "usdt"),
            Some(Level::new(20.0, 9.5)),
            Some(Level::new(22.87, 11.78)),
        );

        // Expected Spreads
        let binance_htx_btc_take_take = 0.0;
        let binance_htx_btc_take_make = 0.0;
        let binance_htx_btc_make_take = 0.0;
        let binance_htx_btc_make_make = 0.0;

        let binance_htx_eth_take_take = 0.0;
        let binance_htx_eth_take_make = 0.0;
        let binance_htx_eth_make_take = 0.0;
        let binance_htx_eth_make_make = 0.0;

        let htx_binance_btc_take_take = 0.0;
        let htx_binance_btc_take_make = 0.0;
        let htx_binance_btc_make_take = 0.0;
        let htx_binance_btc_make_make = 0.0;

        let htx_binance_eth_take_take = 0.0;
        let htx_binance_eth_take_make = 0.0;
        let htx_binance_eth_make_take = 0.0;
        let htx_binance_eth_make_make = 0.0;
    }

    #[test]
    fn test_scanner_swap_existing_data() {
        let mut scanner = test_utils::spot_arb_scanner();
        let mut exchange_data_map = ExchangeMarketDataMap::default();

        // First orderbook update
        let binance_btc_ob = test_utils::market_event_orderbook(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
        );

        scanner.process_orderbook(
            binance_btc_ob.exchange,
            binance_btc_ob.instrument,
            binance_btc_ob.event_data.get_orderbook().unwrap().bids,
            binance_btc_ob.event_data.get_orderbook().unwrap().asks,
        );

        // Second orderbook update
        let new_bids = test_utils::bids_random();
        let new_asks = test_utils::asks_random();

        let binance_btc_ob2 = test_utils::market_event_orderbook2(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            new_bids.clone(),
            new_asks.clone(),
        );

        scanner.process_orderbook(
            binance_btc_ob2.exchange,
            binance_btc_ob2.instrument,
            binance_btc_ob2.event_data.get_orderbook().unwrap().bids,
            binance_btc_ob2.event_data.get_orderbook().unwrap().asks,
        );

        // Expect swapped data
        let mut binance_instrument_map = InstrumentMarketDataMap::default();
        let binance_instrument_data = InstrumentMarketData::new_orderbook(new_bids, new_asks);

        binance_instrument_map
            .0
            .insert(Instrument::new("btc", "usdt"), binance_instrument_data);

        exchange_data_map
            .0
            .insert(ExchangeId::BinanceSpot, binance_instrument_map);

        assert_eq!(scanner.exchange_data, exchange_data_map);

        // Try swap with empty vecs and should not work - I dont think there will ever be empty vecs for MarketEvents
        let binance_btc_ob3 = test_utils::market_event_orderbook2(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            Vec::new(),
            Vec::new(),
        );

        scanner.process_orderbook(
            binance_btc_ob3.exchange,
            binance_btc_ob3.instrument,
            binance_btc_ob3.event_data.get_orderbook().unwrap().bids,
            binance_btc_ob3.event_data.get_orderbook().unwrap().asks,
        );

        assert_eq!(scanner.exchange_data, exchange_data_map);

        // Try swap with only bids data having empty vec - bids data should not swap
        let new_asks2 = test_utils::asks_random();
        let binance_btc_ob4 = test_utils::market_event_orderbook2(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            Vec::new(),
            new_asks2.clone(),
        );

        scanner.process_orderbook(
            binance_btc_ob4.exchange,
            binance_btc_ob4.instrument,
            binance_btc_ob4.event_data.get_orderbook().unwrap().bids,
            binance_btc_ob4.event_data.get_orderbook().unwrap().asks,
        );

        let binance_market_data = exchange_data_map
            .0
            .get_mut(&ExchangeId::BinanceSpot)
            .unwrap()
            .0
            .get_mut(&Instrument::new("btc", "usdt"))
            .unwrap();

        binance_market_data.asks = new_asks2;

        assert_eq!(scanner.exchange_data, exchange_data_map);

        // Try swap with only asks data having empty vec - asks data should not swap
        let new_bids2 = test_utils::bids_random();
        let binance_btc_ob5 = test_utils::market_event_orderbook2(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            new_bids2.clone(),
            Vec::new(),
        );

        scanner.process_orderbook(
            binance_btc_ob5.exchange,
            binance_btc_ob5.instrument,
            binance_btc_ob5.event_data.get_orderbook().unwrap().bids,
            binance_btc_ob5.event_data.get_orderbook().unwrap().asks,
        );

        let binance_market_data = exchange_data_map
            .0
            .get_mut(&ExchangeId::BinanceSpot)
            .unwrap()
            .0
            .get_mut(&Instrument::new("btc", "usdt"))
            .unwrap();

        binance_market_data.bids = new_bids2;

        assert_eq!(scanner.exchange_data, exchange_data_map);
    }

    #[test]
    fn test_scanner_insert() {
        // Init
        let mut scanner = test_utils::spot_arb_scanner();
        let mut exchange_data_map = ExchangeMarketDataMap::default();

        // Binance
        let binance_btc_ob = test_utils::market_event_orderbook(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
        );

        scanner.process_orderbook(
            binance_btc_ob.exchange,
            binance_btc_ob.instrument,
            binance_btc_ob.event_data.get_orderbook().unwrap().bids,
            binance_btc_ob.event_data.get_orderbook().unwrap().asks,
        );

        let mut binance_instrument_map = InstrumentMarketDataMap::default();

        let binance_instrument_data =
            InstrumentMarketData::new_orderbook(test_utils::bids(), test_utils::asks());

        binance_instrument_map
            .0
            .insert(Instrument::new("btc", "usdt"), binance_instrument_data);

        exchange_data_map
            .0
            .insert(ExchangeId::BinanceSpot, binance_instrument_map);

        assert_eq!(scanner.exchange_data, exchange_data_map);

        // Exmo
        let exmo_arb_ob = test_utils::market_event_orderbook(
            ExchangeId::ExmoSpot,
            Instrument::new("arb", "usdt"),
        );

        scanner.process_orderbook(
            exmo_arb_ob.exchange,
            exmo_arb_ob.instrument,
            exmo_arb_ob.event_data.get_orderbook().unwrap().bids,
            exmo_arb_ob.event_data.get_orderbook().unwrap().asks,
        );

        let mut exmo_instrument_map = InstrumentMarketDataMap::default();

        let exmo_instrument_data =
            InstrumentMarketData::new_orderbook(test_utils::bids(), test_utils::asks());

        exmo_instrument_map
            .0
            .insert(Instrument::new("arb", "usdt"), exmo_instrument_data);

        exchange_data_map
            .0
            .insert(ExchangeId::ExmoSpot, exmo_instrument_map);

        assert_eq!(scanner.exchange_data, exchange_data_map);

        // Htx
        let htx_op_ob =
            test_utils::market_event_orderbook(ExchangeId::HtxSpot, Instrument::new("op", "usdt"));

        scanner.process_orderbook(
            htx_op_ob.exchange,
            htx_op_ob.instrument,
            htx_op_ob.event_data.get_orderbook().unwrap().bids,
            htx_op_ob.event_data.get_orderbook().unwrap().asks,
        );

        let mut htx_instrument_map = InstrumentMarketDataMap::default();

        let htx_instrument_data =
            InstrumentMarketData::new_orderbook(test_utils::bids(), test_utils::asks());

        htx_instrument_map
            .0
            .insert(Instrument::new("op", "usdt"), htx_instrument_data);

        exchange_data_map
            .0
            .insert(ExchangeId::HtxSpot, htx_instrument_map);

        assert_eq!(scanner.exchange_data, exchange_data_map);
    }

    #[test]
    fn test_vecdequeuetime_new_creation() {
        let time = Utc::now();
        let value = 42;
        let queue = VecDequeTime::new(time, value);

        assert_eq!(queue.data.len(), 1);
        assert_eq!(queue.window, Duration::minutes(10));

        if let Some((stored_time, stored_value)) = queue.data.front() {
            assert_eq!(*stored_time, time);
            assert_eq!(*stored_value, value);
        } else {
            panic!("Queue should contain one element");
        }
    }

    #[test]
    fn test_vecdequeuetime_push_within_window() {
        let start_time = Utc.timestamp_opt(0, 0).unwrap();
        let mut queue = VecDequeTime::new(start_time, 1);

        // Add values 5 minutes apart
        queue.push(start_time + Duration::minutes(5), 2);
        queue.push(start_time + Duration::minutes(8), 3);

        assert_eq!(queue.data.len(), 3);

        // Verify all values are present in order
        let values: Vec<i32> = queue.data.iter().map(|(_, v)| *v).collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_vecdequeuetime_cleanup_old_data() {
        let start_time = Utc.timestamp_opt(0, 0).unwrap();
        let mut queue = VecDequeTime::new(start_time, 1);

        // Add some values within the window
        queue.push(start_time + Duration::minutes(3), 2);
        queue.push(start_time + Duration::minutes(6), 3);
        assert_eq!(queue.data.len(), 3);

        // Push a value that's 15 minutes after start, should trigger cleanup
        queue.push(start_time + Duration::minutes(15), 4);

        // First two values should be removed (0 and 3 minutes), leaving only
        // the 6 minute and 15 minute values
        assert_eq!(queue.data.len(), 2);

        let remaining_values: Vec<i32> = queue.data.iter().map(|(_, v)| *v).collect();
        assert_eq!(remaining_values, vec![3, 4]);
    }

    #[test]
    fn test_vecdequeuetime_multiple_cleanups() {
        let start_time = Utc.timestamp_opt(0, 0).unwrap();
        let mut queue = VecDequeTime::new(start_time, 1);

        // Add values at different times
        for i in 1..=5 {
            queue.push(start_time + Duration::minutes(i * 2), i + 1);
        }
        assert_eq!(queue.data.len(), 6); // Original + 5 new values

        // Push value 21 minutes later, should clear all previous values
        queue.push(start_time + Duration::minutes(21), 7);

        assert_eq!(queue.data.len(), 1);

        if let Some((_, value)) = queue.data.back() {
            assert_eq!(*value, 7);
        } else {
            panic!("Queue should contain the last value");
        }
    }

    #[test]
    fn test_spread_sorted() {
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

        spread_map.insert(k1.clone(), s1);
        spread_map.insert(k2.clone(), s2);
        spread_map.insert(k3.clone(), s3);
        spread_map.insert(k4.clone(), s4);

        let result = spread_map.snapshot();
        let expected = vec![
            (s3, k3.clone()),
            (s1, k1.clone()),
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
            (s1, k1.clone()),
            (s2, k2.clone()),
        ];
        assert_eq!(result, expected);
    }
}
