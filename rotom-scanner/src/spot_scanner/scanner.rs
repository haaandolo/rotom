use chrono::{DateTime, Utc};
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
use serde::Serialize;
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tracing::warn;

use crate::server::{
    server_channels::ScannerHttpChannel, SpotArbScannerHttpRequests, SpotArbScannerHttpResponse,
};

use super::core_types::{
    AverageTradeInfo, ExchangeMarketDataMap, InstrumentMarketData, LatestSpreads, NetworkStatusMap,
    SpreadHistory, SpreadKey, SpreadsSorted,
};

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
// Spread Change
/*----- */
#[derive(Debug)]
pub struct SpreadsCalculated {
    pub sc_exchange: ExchangeId,
    pub other_exchange: ExchangeId,
    pub instrument: Instrument,
    pub spread_array: [Option<f64>; 4],
}

impl SpreadsCalculated {
    pub fn new(
        sc_exchange: ExchangeId,
        other_exchange: ExchangeId,
        instrument: Instrument,
        spread_array: [Option<f64>; 4],
    ) -> Self {
        Self {
            sc_exchange,
            other_exchange,
            instrument,
            spread_array,
        }
    }
}

#[derive(Debug)]
enum SpreadChangeProcess {
    SpreadChange(SpreadChange),
    SpreadsCalculated(SpreadsCalculated),
}

/*----- */
// Spread Response - Response sent back from http request
/*----- */
#[derive(Debug, Serialize)]
pub struct SpreadResponse {
    pub base_exchange: ExchangeId,
    pub quote_exchange: ExchangeId,
    pub instrument: Instrument,
    pub spreads: LatestSpreads,
    pub base_exchange_avg_trade_info: AverageTradeInfo,
    pub quote_exchange_avg_trade_info: AverageTradeInfo,
    pub base_exchange_network_info: Option<NetworkSpecData>,
    pub quote_exchange_network_info: Option<NetworkSpecData>,
    pub base_exchange_trades_ws_is_connected: bool,
    pub base_exchange_orderbook_ws_is_connected: bool,
    pub quote_exchange_trades_ws_is_connected: bool,
    pub quote_exchange_orderbook_ws_is_connected: bool,
}

/*----- */
// Spread History Response - Response sent back from http request
/*----- */
#[derive(Debug, Serialize)]
pub struct SpreadHistoryResponse {
    pub base_exchange: ExchangeId,
    pub quote_exchange: ExchangeId,
    pub instrument: Instrument,
    pub spread_history: SpreadHistory,
}

/*----- */
// Spot Arb Scanner
/*----- */
#[derive(Debug)]
pub struct SpotArbScanner {
    exchange_data: ExchangeMarketDataMap,
    network_status: NetworkStatusMap,
    spreads_sorted: SpreadsSorted,
    spread_change_queue: VecDeque<SpreadChangeProcess>,
    network_status_stream: mpsc::UnboundedReceiver<NetworkSpecs>,
    market_data_stream: mpsc::UnboundedReceiver<MarketEvent<DataKind>>,
    http_channel: ScannerHttpChannel,
}

impl SpotArbScanner {
    pub fn new(
        network_status_stream: mpsc::UnboundedReceiver<NetworkSpecs>,
        market_data_stream: mpsc::UnboundedReceiver<MarketEvent<DataKind>>,
        http_channel: ScannerHttpChannel,
    ) -> Self {
        Self {
            exchange_data: ExchangeMarketDataMap::default(),
            network_status: NetworkStatusMap::default(),
            spreads_sorted: SpreadsSorted::default(),
            spread_change_queue: VecDeque::with_capacity(10),
            network_status_stream,
            market_data_stream,
            http_channel,
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
                        self.spread_change_queue
                            .push_back(SpreadChangeProcess::SpreadChange(new_bba));
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

    fn sort_option_array(mut arr: [Option<f64>; 4]) -> [Option<f64>; 4] {
        // Sort the array, placing None values at the end
        arr.sort_by(|a, b| match (a, b) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, _) => std::cmp::Ordering::Greater,
            (_, None) => std::cmp::Ordering::Less,
            (Some(x), Some(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        });
        arr
    }

    fn process_spread_change(&mut self, spread_change: SpreadChange) {
        for (exchange, market_data_map) in &self.exchange_data.0 {
            if *exchange != spread_change.exchange {
                if let Some(market_data) = market_data_map.0.get(&spread_change.instrument) {
                    // We assume the exchange associated with the spread change is the buy exchange, so this is in the denominator.
                    // Hence, the sell exchange is the exchange correspoding to the market_data
                    let mut spread_array = [None; 4];

                    if let Some(spread_change_ask) = spread_change.ask {
                        // Calculate the spreads if best ask level has changed
                        if !market_data.bids.is_empty() {
                            let take_take =
                                (market_data.bids[0].price / spread_change_ask.price) - 1.0;
                            spread_array[0] = Some(take_take)
                        }

                        if !market_data.asks.is_empty() {
                            let take_make =
                                (market_data.asks[0].price / spread_change_ask.price) - 1.0;
                            spread_array[1] = Some(take_make)
                        }
                    }

                    if let Some(spread_change_bid) = spread_change.bid {
                        // Calculate the spreads if best bid level has changed
                        if !market_data.bids.is_empty() {
                            let make_take =
                                (market_data.bids[0].price / spread_change_bid.price) - 1.0;
                            spread_array[2] = Some(make_take)
                        }

                        if !market_data.asks.is_empty() {
                            let make_make =
                                (market_data.asks[0].price / spread_change_bid.price) - 1.0;
                            spread_array[3] = Some(make_make)
                        }
                    }

                    let spread_change_calculated = SpreadsCalculated::new(
                        spread_change.exchange,
                        *exchange,
                        spread_change.instrument.clone(),
                        spread_array,
                    );

                    self.spread_change_queue
                        .push_back(SpreadChangeProcess::SpreadsCalculated(
                            spread_change_calculated,
                        ));
                }
            }
        }
    }

    fn process_calculated_spreads(&mut self, time: DateTime<Utc>, spreads: SpreadsCalculated) {
        // Update spread history of spread change exchange
        if let Some(sc_market_data_map) = self.exchange_data.0.get_mut(&spreads.sc_exchange) {
            if let Some(sc_market_data) = sc_market_data_map.0.get_mut(&spreads.instrument) {
                sc_market_data
                    .spreads
                    .0
                    .entry(spreads.other_exchange)
                    .and_modify(|spread_history| {
                        spread_history.insert(time, spreads.spread_array);
                    })
                    .or_insert_with(|| {
                        let mut spread_default = SpreadHistory::default();
                        spread_default.insert(time, spreads.spread_array);
                        spread_default
                    });
            }
        }

        // Get max spread - index into 3 as it is the last value of the array
        let max_spread = Self::sort_option_array(spreads.spread_array)[3];

        // Insert into spreads_sorted
        if let Some(max_spread) = max_spread {
            if max_spread > 0.0 {
                self.spreads_sorted.insert(
                    SpreadKey::new(
                        spreads.sc_exchange,
                        spreads.other_exchange,
                        spreads.instrument.clone(),
                    ),
                    max_spread,
                );
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

    fn process_get_top_spread_request(&mut self) {
        // Get top spreads
        let top_spreads = self
            .spreads_sorted
            .snapshot()
            .into_iter()
            .map(|(_, spread_key)| {
                // Note: unwraps here should never fail as before this step everything should have a value
                let (base_exchange, quote_exchange) = spread_key.exchanges;
                let instrument = spread_key.instrument;
                let coin = Coin::new(instrument.base.as_str());

                // Get relevant base exchange info
                let base_exchange_data = self
                    .exchange_data
                    .0
                    .get(&base_exchange)
                    .unwrap() // Should never fail
                    .0
                    .get(&instrument)
                    .unwrap(); // Should never fail

                let spreads = base_exchange_data
                    .spreads
                    .0
                    .get(&quote_exchange)
                    .unwrap() // Should never fail
                    .latest_spreads;

                let base_exchange_network_info = self
                    .network_status
                    .0
                    .get(&(base_exchange, coin.clone()))
                    .cloned();

                let base_exchange_trades_ws_is_connected =
                    base_exchange_data.trades_ws_is_connected;
                let base_exchange_orderbook_ws_is_connected =
                    base_exchange_data.orderbook_ws_is_connected;
                let base_exchange_avg_trade_info = base_exchange_data.get_average_trades();

                // Get relevant quote exchange info
                let quote_exchange_data = self
                    .exchange_data
                    .0
                    .get(&quote_exchange)
                    .unwrap() // Should never fail
                    .0
                    .get(&instrument)
                    .unwrap(); // Should never fail

                let quote_exchange_network_info =
                    self.network_status.0.get(&(quote_exchange, coin)).cloned();

                let quote_exchange_trades_ws_is_connected =
                    quote_exchange_data.trades_ws_is_connected;
                let quote_exchange_orderbook_ws_is_connected =
                    quote_exchange_data.orderbook_ws_is_connected;
                let quote_exchange_avg_trade_info = quote_exchange_data.get_average_trades();

                SpreadResponse {
                    base_exchange,
                    quote_exchange,
                    instrument,
                    spreads,
                    base_exchange_avg_trade_info,
                    quote_exchange_avg_trade_info,
                    base_exchange_network_info,
                    quote_exchange_network_info,
                    base_exchange_trades_ws_is_connected,
                    base_exchange_orderbook_ws_is_connected,
                    quote_exchange_trades_ws_is_connected,
                    quote_exchange_orderbook_ws_is_connected,
                }
            })
            .collect::<Vec<SpreadResponse>>();

        // Send response via channel
        let _ = self
            .http_channel
            .http_response_tx
            .send(SpotArbScannerHttpResponse::GetTopSpreads(top_spreads));
    }

    fn process_get_spread_history_request(
        &mut self,
        base_exchange: ExchangeId,
        quote_exchange: ExchangeId,
        instrument: Instrument,
    ) {
        let base_exchange_spread_history = self
            .exchange_data
            .0
            .get(&base_exchange)
            .and_then(|base_exchange_instrument_market_data| {
                base_exchange_instrument_market_data.0.get(&instrument)
            })
            .and_then(|base_exchange_spread_history| {
                base_exchange_spread_history.spreads.0.get(&quote_exchange)
            });

        match base_exchange_spread_history {
            Some(base_exchange_spread_history_response) => {
                let _ = self.http_channel.http_response_tx.send(
                    SpotArbScannerHttpResponse::GetSpreadHistory(Box::new(SpreadHistoryResponse {
                        base_exchange,
                        quote_exchange,
                        instrument,
                        spread_history: base_exchange_spread_history_response.clone(),
                    })),
                );
            }
            None => {
                let _ = self.http_channel.http_response_tx.send(
                    SpotArbScannerHttpResponse::CouldNotFindSpreadHistory {
                        base_exchange,
                        quote_exchange,
                        instrument,
                    },
                );
            }
        }
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
                            action = "Breaking Spot Arb Scanner loop",
                        );
                        break 'spot_arb_scanner;
                    }
                }
            }

            // Process http requests
            match self.http_channel.http_request_rx.try_recv() {
                Ok(request) => match request {
                    SpotArbScannerHttpRequests::GetTopSpreads => {
                        self.process_get_top_spread_request()
                    }
                    SpotArbScannerHttpRequests::GetSpreadHistory((
                        base_exchange,
                        quote_exchange,
                        instrument,
                    )) => self.process_get_spread_history_request(
                        base_exchange,
                        quote_exchange,
                        instrument,
                    ),
                },
                Err(error) => {
                    if error == mpsc::error::TryRecvError::Disconnected {
                        warn!(
                            message = "Http request channel has disconnected",
                            action = "Breaking Spot Arb Scanner loop",
                        );
                        break 'spot_arb_scanner;
                    }
                }
            }

            // Process market data update
            match self.market_data_stream.try_recv() {
                Ok(market_data) => match market_data.event_data {
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
                },
                Err(error) => {
                    if error == mpsc::error::TryRecvError::Disconnected {
                        warn!(
                            message = "Network status stream for spot arb scanner has disconnected",
                            action = "Breaking Spot Arb Scanner loop",
                        );
                        break 'spot_arb_scanner;
                    }
                }
            };

            // println!("######### \n {:#?}", self.market_data_stream.len());

            // Process spreads - Have to do queue with enums to avoid borrowing rule errors
            while let Some(spread_change_process) = self.spread_change_queue.pop_front() {
                match spread_change_process {
                    SpreadChangeProcess::SpreadChange(spread_change) => {
                        self.process_spread_change(spread_change)
                    }
                    SpreadChangeProcess::SpreadsCalculated(calculated_spreads) => {
                        self.process_calculated_spreads(Utc::now(), calculated_spreads);
                    }
                }

                // println!("{:#?}", self.spreads_sorted);
            }
        }
    }
}

/*----- */
// Test
/*----- */
#[cfg(test)]
mod test {
    use chrono::{Duration, TimeZone};
    use rotom_data::{
        model::network_info::{ChainSpecs, NetworkSpecData},
        shared::subscription_models::Coin,
    };
    use std::collections::HashMap;

    use crate::spot_scanner::{
        core_types::{InstrumentMarketDataMap, VecDequeTime},
        mock_data::test_utils,
    };

    use super::*;

    struct TestCase {
        name: &'static str,
        old_bid: Level,
        old_ask: Level,
        new_bid: Level,
        new_ask: Level,
        expected: Option<SpreadChange>,
    }

    #[test]
    fn test_network_status() {
        // Create a scanner instance
        let mut scanner = test_utils::spot_arb_scanner(); // Adjust based on your constructor

        // Create initial network status
        let mut initial_network_specs = HashMap::new();
        let initial_chain_spec = ChainSpecs {
            chain_name: "Ethereum".to_string(),
            fee_is_fixed: true,
            fees: 0.001,
            can_deposit: true,
            can_withdraw: true,
        };

        initial_network_specs.insert(
            (ExchangeId::BinanceSpot, Coin(String::from("btc"))),
            NetworkSpecData(vec![initial_chain_spec]),
        );
        scanner.network_status = NetworkStatusMap(initial_network_specs);

        // Create new network status to process
        let mut new_network_specs = HashMap::new();

        // Updated specs for existing entry
        let updated_chain_spec = ChainSpecs {
            chain_name: "Ethereum".to_string(),
            fee_is_fixed: false, // Changed
            fees: 0.002,         // Changed
            can_deposit: true,
            can_withdraw: false, // Changed
        };
        new_network_specs.insert(
            (ExchangeId::BinanceSpot, Coin(String::from("btc"))),
            NetworkSpecData(vec![updated_chain_spec.clone()]),
        );

        // New entry
        let new_chain_spec = ChainSpecs {
            chain_name: "Solana".to_string(),
            fee_is_fixed: true,
            fees: 0.0001,
            can_deposit: true,
            can_withdraw: true,
        };
        new_network_specs.insert(
            (ExchangeId::HtxSpot, Coin(String::from("sol"))),
            NetworkSpecData(vec![new_chain_spec.clone()]),
        );

        // Process the new network status
        scanner.process_network_status(NetworkSpecs(new_network_specs));

        // Verify the results
        let network_status = &scanner.network_status.0;

        // Check updated entry
        let binance_btc = network_status
            .get(&(ExchangeId::BinanceSpot, Coin(String::from("btc"))))
            .unwrap();

        assert_eq!(binance_btc.0.len(), 1);
        assert_eq!(binance_btc.0[0].chain_name, "Ethereum");
        assert!(!binance_btc.0[0].fee_is_fixed);
        assert_eq!(binance_btc.0[0].fees, 0.002);
        assert!(!binance_btc.0[0].can_withdraw);

        // Check new entry
        let htx_sol = network_status
            .get(&(ExchangeId::HtxSpot, Coin(String::from("sol"))))
            .unwrap();

        assert_eq!(htx_sol.0.len(), 1);
        assert_eq!(htx_sol.0[0].chain_name, "Solana");
        assert!(htx_sol.0[0].fee_is_fixed);
        assert_eq!(htx_sol.0[0].fees, 0.0001);
        assert!(htx_sol.0[0].can_withdraw);
    }

    #[test]
    fn test_ws_status() {
        // Init
        let mut scanner = test_utils::spot_arb_scanner();

        // New orderbook connection success notification arrives for new exchange instrument combo
        scanner.process_ws_status(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            WsStatus::Connected(EventKind::OrderBook),
        );

        let result = scanner
            .exchange_data
            .0
            .get(&ExchangeId::BinanceSpot)
            .unwrap()
            .0
            .get(&Instrument::new("btc", "usdt"))
            .unwrap();

        let expected = InstrumentMarketData {
            orderbook_ws_is_connected: true,
            ..InstrumentMarketData::default()
        };

        assert_eq!(result, &expected);

        // Trade connection success notification arrives
        scanner.process_ws_status(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            WsStatus::Connected(EventKind::Trade),
        );

        let result = scanner
            .exchange_data
            .0
            .get(&ExchangeId::BinanceSpot)
            .unwrap()
            .0
            .get(&Instrument::new("btc", "usdt"))
            .unwrap();

        let expected = InstrumentMarketData {
            orderbook_ws_is_connected: true,
            trades_ws_is_connected: true,
            ..InstrumentMarketData::default()
        };

        assert_eq!(result, &expected);

        // Disconnection connectoin notification for orderbook arrives
        scanner.process_ws_status(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            WsStatus::Disconnected(EventKind::OrderBook),
        );

        let result = scanner
            .exchange_data
            .0
            .get(&ExchangeId::BinanceSpot)
            .unwrap()
            .0
            .get(&Instrument::new("btc", "usdt"))
            .unwrap();

        let expected = InstrumentMarketData {
            orderbook_ws_is_connected: false,
            trades_ws_is_connected: true,
            ..InstrumentMarketData::default()
        };

        assert_eq!(result, &expected);

        // New trade connection success notification arrives for new exchange instrument combo
        scanner.process_ws_status(
            ExchangeId::HtxSpot,
            Instrument::new("eth", "usdt"),
            WsStatus::Connected(EventKind::Trade),
        );

        let result = scanner
            .exchange_data
            .0
            .get(&ExchangeId::HtxSpot)
            .unwrap()
            .0
            .get(&Instrument::new("eth", "usdt"))
            .unwrap();

        let expected = InstrumentMarketData {
            trades_ws_is_connected: true,
            ..InstrumentMarketData::default()
        };

        assert_eq!(result, &expected);

        // Orderbook connection success arrives
        scanner.process_ws_status(
            ExchangeId::HtxSpot,
            Instrument::new("eth", "usdt"),
            WsStatus::Connected(EventKind::OrderBook),
        );

        let result = scanner
            .exchange_data
            .0
            .get(&ExchangeId::HtxSpot)
            .unwrap()
            .0
            .get(&Instrument::new("eth", "usdt"))
            .unwrap();

        let expected = InstrumentMarketData {
            orderbook_ws_is_connected: true,
            trades_ws_is_connected: true,
            ..InstrumentMarketData::default()
        };

        assert_eq!(result, &expected);
    }

    #[test]
    fn test_did_bba_change() {
        let exchange = ExchangeId::BinanceSpot;
        let instrument = Instrument::new("BTC", "USDT");

        let test_cases = vec![
            // Case 1: No changes
            TestCase {
                name: "no_changes",
                old_bid: Level {
                    price: 50000.0,
                    size: 1.0,
                },
                old_ask: Level {
                    price: 50100.0,
                    size: 1.0,
                },
                new_bid: Level {
                    price: 50000.0,
                    size: 1.0,
                },
                new_ask: Level {
                    price: 50100.0,
                    size: 1.0,
                },
                expected: None,
            },
            // Case 2: Only bid changed
            TestCase {
                name: "bid_change_only",
                old_bid: Level {
                    price: 50000.0,
                    size: 1.0,
                },
                old_ask: Level {
                    price: 50100.0,
                    size: 1.0,
                },
                new_bid: Level {
                    price: 50050.0,
                    size: 1.5,
                },
                new_ask: Level {
                    price: 50100.0,
                    size: 1.0,
                },
                expected: Some(SpreadChange::new_bid(
                    exchange,
                    instrument.clone(),
                    Level {
                        price: 50050.0,
                        size: 1.5,
                    },
                )),
            },
            // Case 3: Only ask changed
            TestCase {
                name: "ask_change_only",
                old_bid: Level {
                    price: 50000.0,
                    size: 1.0,
                },
                old_ask: Level {
                    price: 50100.0,
                    size: 1.0,
                },
                new_bid: Level {
                    price: 50000.0,
                    size: 1.0,
                },
                new_ask: Level {
                    price: 50090.0,
                    size: 2.0,
                },
                expected: Some(SpreadChange::new_ask(
                    exchange,
                    instrument.clone(),
                    Level {
                        price: 50090.0,
                        size: 2.0,
                    },
                )),
            },
            // Case 4: Both bid and ask changed
            TestCase {
                name: "both_changed",
                old_bid: Level {
                    price: 50000.0,
                    size: 1.0,
                },
                old_ask: Level {
                    price: 50100.0,
                    size: 1.0,
                },
                new_bid: Level {
                    price: 50050.0,
                    size: 1.5,
                },
                new_ask: Level {
                    price: 50090.0,
                    size: 2.0,
                },
                expected: Some({
                    let mut change = SpreadChange::new_bid(
                        exchange,
                        instrument.clone(),
                        Level {
                            price: 50050.0,
                            size: 1.5,
                        },
                    );
                    change.add_ask(Level {
                        price: 50090.0,
                        size: 2.0,
                    });
                    change
                }),
            },
        ];

        for case in test_cases {
            let result = SpotArbScanner::did_bba_change(
                exchange,
                instrument.clone(),
                case.old_bid,
                case.old_ask,
                case.new_bid,
                case.new_ask,
            );

            match (result, case.expected) {
                (None, None) => (),
                (Some(result_change), Some(expected_change)) => {
                    assert_eq!(
                        result_change.exchange, expected_change.exchange,
                        "Failed on case {}: exchange mismatch",
                        case.name
                    );
                    assert_eq!(
                        result_change.instrument, expected_change.instrument,
                        "Failed on case {}: instrument mismatch",
                        case.name
                    );
                    assert_eq!(
                        result_change.bid, expected_change.bid,
                        "Failed on case {}: bids mismatch",
                        case.name
                    );
                    assert_eq!(
                        result_change.ask, expected_change.ask,
                        "Failed on case {}: asks mismatch",
                        case.name
                    );
                }
                _ => panic!(
                    "Failed on case {}: result and expected variants don't match",
                    case.name
                ),
            }
        }
    }

    #[test]
    fn test_scanner_spread() {
        /*----- */
        // Init
        /*----- */
        let mut scanner = test_utils::spot_arb_scanner();
        let time = Utc::now();

        /*----- */
        // Init Binance market data state
        /*----- */
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

        /*----- */
        // Init Htx market data state
        /*----- */
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

        /*----- */
        // New orderbook snapshot arrives for binance triggering a spread change
        /*----- */
        let binance_btc_spread_change = test_utils::market_event_orderbook2(
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
            vec![Level::new(12.22, 9.0)],
            vec![Level::new(15.12, 18.03)],
        );

        scanner.process_orderbook(
            binance_btc_spread_change.exchange,
            binance_btc_spread_change.instrument,
            binance_btc_spread_change
                .event_data
                .get_orderbook()
                .unwrap()
                .bids,
            binance_btc_spread_change
                .event_data
                .get_orderbook()
                .unwrap()
                .asks,
        );

        let binance_eth_spread_change = test_utils::market_event_orderbook2(
            ExchangeId::BinanceSpot,
            Instrument::new("eth", "usdt"),
            vec![Level::new(20.78, 9.255)],
            vec![Level::new(21.92, 18.78)],
        );

        scanner.process_orderbook(
            binance_eth_spread_change.exchange,
            binance_eth_spread_change.instrument,
            binance_eth_spread_change
                .event_data
                .get_orderbook()
                .unwrap()
                .bids,
            binance_eth_spread_change
                .event_data
                .get_orderbook()
                .unwrap()
                .asks,
        );

        // Manually trigger SpreadChangeProcess for Binance
        let binance_spread_change = std::mem::take(&mut scanner.spread_change_queue);
        let mut binance_spread_changes = binance_spread_change
            .into_iter()
            .filter_map(|spread_change_process| {
                if let SpreadChangeProcess::SpreadChange(spread_change) = spread_change_process {
                    Some(spread_change)
                } else {
                    None
                }
            })
            .collect::<Vec<SpreadChange>>();

        scanner.process_spread_change(binance_spread_changes.remove(0));
        scanner.process_spread_change(binance_spread_changes.remove(0));

        let binance_spread_changes_calculated = std::mem::take(&mut scanner.spread_change_queue);
        let mut binance_spread_changes_calculated = binance_spread_changes_calculated
            .into_iter()
            .filter_map(|spread_change_process| {
                if let SpreadChangeProcess::SpreadsCalculated(spread_change_calculated) =
                    spread_change_process
                {
                    Some(spread_change_calculated)
                } else {
                    None
                }
            })
            .collect::<Vec<SpreadsCalculated>>();

        scanner.process_calculated_spreads(time, binance_spread_changes_calculated.remove(0));
        scanner.process_calculated_spreads(time, binance_spread_changes_calculated.remove(0));

        // Expected spreads for Binance
        let binance_htx_btc_take_take = (13.5 / 15.12) - 1.0; // -0.107
        let binance_htx_btc_take_make = (14.5 / 15.12) - 1.0; // -0.0410
        let binance_htx_btc_make_take = (13.5 / 12.22) - 1.0; // 0.1047 --> doesn't get inserted as not the best spread for given instrument
        let binance_htx_btc_make_make = (14.5 / 12.22) - 1.0; // 0.1866 --> 1

        let binance_htx_eth_take_take = (19.22 / 21.92) - 1.0; // -0.1231
        let binance_htx_eth_take_make = (20.11 / 21.92) - 1.0; // -0.0826
        let binance_htx_eth_make_take = (19.22 / 20.78) - 1.0; // -0.0751
        let binance_htx_eth_make_make = (20.11 / 20.78) - 1.0; // -0.3224

        // Check Binance btc spread history
        let mut binance_btc_spread_history = SpreadHistory::default();

        binance_btc_spread_history.latest_spreads.take_take = binance_htx_btc_take_take;
        binance_btc_spread_history
            .take_take
            .push(time, binance_htx_btc_take_take);

        binance_btc_spread_history.latest_spreads.take_make = binance_htx_btc_take_make;
        binance_btc_spread_history
            .take_make
            .push(time, binance_htx_btc_take_make);

        binance_btc_spread_history.latest_spreads.make_take = binance_htx_btc_make_take;
        binance_btc_spread_history
            .make_take
            .push(time, binance_htx_btc_make_take);

        binance_btc_spread_history.latest_spreads.make_make = binance_htx_btc_make_make;
        binance_btc_spread_history
            .make_make
            .push(time, binance_htx_btc_make_make);

        let result = scanner
            .exchange_data
            .0
            .get(&ExchangeId::BinanceSpot)
            .unwrap()
            .0
            .get(&Instrument::new("btc", "usdt"))
            .unwrap()
            .spreads
            .0
            .get(&ExchangeId::HtxSpot)
            .unwrap();
        let expected = &binance_btc_spread_history;
        assert_eq!(result, expected);

        // Check Binance eth spread history
        let mut binance_eth_spread_history = SpreadHistory::default();

        binance_eth_spread_history.latest_spreads.take_take = binance_htx_eth_take_take;
        binance_eth_spread_history
            .take_take
            .push(time, binance_htx_eth_take_take);

        binance_eth_spread_history.latest_spreads.take_make = binance_htx_eth_take_make;
        binance_eth_spread_history
            .take_make
            .push(time, binance_htx_eth_take_make);

        binance_eth_spread_history.latest_spreads.make_take = binance_htx_eth_make_take;
        binance_eth_spread_history
            .make_take
            .push(time, binance_htx_eth_make_take);

        binance_eth_spread_history.latest_spreads.make_make = binance_htx_eth_make_make;
        binance_eth_spread_history
            .make_make
            .push(time, binance_htx_eth_make_make);

        let result = scanner
            .exchange_data
            .0
            .get(&ExchangeId::BinanceSpot)
            .unwrap()
            .0
            .get(&Instrument::new("eth", "usdt"))
            .unwrap()
            .spreads
            .0
            .get(&ExchangeId::HtxSpot)
            .unwrap();
        let expected = &binance_eth_spread_history;
        assert_eq!(expected, result);

        /*----- */
        // New orderbook snapshot arrives for binance triggering a spread change
        /*----- */
        // Process Htx spread change
        let htx_btc_spread_change = test_utils::market_event_orderbook2(
            ExchangeId::HtxSpot,
            Instrument::new("btc", "usdt"),
            vec![Level::new(13.12, 3.0)],
            vec![Level::new(18.02, 19.03)],
        );

        scanner.process_orderbook(
            htx_btc_spread_change.exchange,
            htx_btc_spread_change.instrument,
            htx_btc_spread_change
                .event_data
                .get_orderbook()
                .unwrap()
                .bids,
            htx_btc_spread_change
                .event_data
                .get_orderbook()
                .unwrap()
                .asks,
        );

        let htx_eth_spread_change = test_utils::market_event_orderbook2(
            ExchangeId::HtxSpot,
            Instrument::new("eth", "usdt"),
            vec![Level::new(20.0, 9.5)],
            vec![Level::new(22.87, 11.78)],
        );

        scanner.process_orderbook(
            htx_eth_spread_change.exchange,
            htx_eth_spread_change.instrument,
            htx_eth_spread_change
                .event_data
                .get_orderbook()
                .unwrap()
                .bids,
            htx_eth_spread_change
                .event_data
                .get_orderbook()
                .unwrap()
                .asks,
        );

        // Manually trigger SpreadChangeProcess for Htx
        let htx_spread_change = std::mem::take(&mut scanner.spread_change_queue);
        let mut htx_spread_changes = htx_spread_change
            .into_iter()
            .filter_map(|spread_change_process| {
                if let SpreadChangeProcess::SpreadChange(spread_change) = spread_change_process {
                    Some(spread_change)
                } else {
                    None
                }
            })
            .collect::<Vec<SpreadChange>>();

        scanner.process_spread_change(htx_spread_changes.remove(0));
        scanner.process_spread_change(htx_spread_changes.remove(0));

        let htx_spread_changes_calculated = std::mem::take(&mut scanner.spread_change_queue);
        let mut htx_spread_changes_calculated = htx_spread_changes_calculated
            .into_iter()
            .filter_map(|spread_change_process| {
                if let SpreadChangeProcess::SpreadsCalculated(spread_change_calculated) =
                    spread_change_process
                {
                    Some(spread_change_calculated)
                } else {
                    None
                }
            })
            .collect::<Vec<SpreadsCalculated>>();

        scanner.process_calculated_spreads(time, htx_spread_changes_calculated.remove(0));
        scanner.process_calculated_spreads(time, htx_spread_changes_calculated.remove(0));

        // Expected spread for htx
        let htx_binance_btc_take_take = (12.22 / 18.02) - 1.0;
        let htx_binance_btc_take_make = (15.12 / 18.02) - 1.0;
        let htx_binance_btc_make_take = (12.22 / 13.12) - 1.0;
        let htx_binance_btc_make_make = (15.12 / 13.12) - 1.0; // 0.1524

        let htx_binance_eth_take_take = (20.78 / 22.87) - 1.0;
        let htx_binance_eth_take_make = (21.92 / 22.87) - 1.0;
        let htx_binance_eth_make_take = (20.78 / 20.0) - 1.0;
        let htx_binance_eth_make_make = (21.92 / 20.0) - 1.0; // 0.096

        // Check Htx btc spread history
        let mut htx_btc_spread_history = SpreadHistory::default();

        htx_btc_spread_history.latest_spreads.take_take = htx_binance_btc_take_take;
        htx_btc_spread_history
            .take_take
            .push(time, htx_binance_btc_take_take);

        htx_btc_spread_history.latest_spreads.take_make = htx_binance_btc_take_make;
        htx_btc_spread_history
            .take_make
            .push(time, htx_binance_btc_take_make);

        htx_btc_spread_history.latest_spreads.make_take = htx_binance_btc_make_take;
        htx_btc_spread_history
            .make_take
            .push(time, htx_binance_btc_make_take);

        htx_btc_spread_history.latest_spreads.make_make = htx_binance_btc_make_make;
        htx_btc_spread_history
            .make_make
            .push(time, htx_binance_btc_make_make);

        let result = scanner
            .exchange_data
            .0
            .get(&ExchangeId::HtxSpot)
            .unwrap()
            .0
            .get(&Instrument::new("btc", "usdt"))
            .unwrap()
            .spreads
            .0
            .get(&ExchangeId::BinanceSpot)
            .unwrap();
        let expected = &htx_btc_spread_history;
        assert_eq!(result, expected);

        // Check eth btc spread history
        let mut htx_eth_spread_history = SpreadHistory::default();

        htx_eth_spread_history.latest_spreads.take_take = htx_binance_eth_take_take;
        htx_eth_spread_history
            .take_take
            .push(time, htx_binance_eth_take_take);

        htx_eth_spread_history.latest_spreads.take_make = htx_binance_eth_take_make;
        htx_eth_spread_history
            .take_make
            .push(time, htx_binance_eth_take_make);

        htx_eth_spread_history.latest_spreads.make_take = htx_binance_eth_make_take;
        htx_eth_spread_history
            .make_take
            .push(time, htx_binance_eth_make_take);

        htx_eth_spread_history.latest_spreads.make_make = htx_binance_eth_make_make;
        htx_eth_spread_history
            .make_make
            .push(time, htx_binance_eth_make_make);

        let result = scanner
            .exchange_data
            .0
            .get(&ExchangeId::HtxSpot)
            .unwrap()
            .0
            .get(&Instrument::new("eth", "usdt"))
            .unwrap()
            .spreads
            .0
            .get(&ExchangeId::BinanceSpot)
            .unwrap();
        let expected = &htx_eth_spread_history;
        assert_eq!(result, expected);

        /*----- */
        // Expected spread
        /*----- */
        let mut expected_spread = SpreadsSorted::new();

        let binance_htx_btc = SpreadKey::new(
            ExchangeId::BinanceSpot,
            ExchangeId::HtxSpot,
            Instrument::new("btc", "usdt"),
        );

        let htx_binance_btc = SpreadKey::new(
            ExchangeId::HtxSpot,
            ExchangeId::BinanceSpot,
            Instrument::new("btc", "usdt"),
        );

        let htx_binance_eth = SpreadKey::new(
            ExchangeId::HtxSpot,
            ExchangeId::BinanceSpot,
            Instrument::new("eth", "usdt"),
        );

        expected_spread.insert(binance_htx_btc, binance_htx_btc_make_make);
        expected_spread.insert(htx_binance_btc, htx_binance_btc_make_make);
        expected_spread.insert(htx_binance_eth, htx_binance_eth_make_make);

        let result = scanner.spreads_sorted.snapshot();
        let expected = expected_spread.snapshot();
        assert_eq!(result, expected);
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
