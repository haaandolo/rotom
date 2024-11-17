use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Duration, Utc};
use rotom_data::{
    assets::level::Level,
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
    MarketMeta,
};

use super::{Decision, Signal, SignalGenerator, SignalStrength};

/*----- */
// Spread strategy
/*----- */
#[derive(Debug)]
pub struct SpreadStategy {
    pub liquid_exchange: SpreadStategyExchange,
    pub illiquid_exchange: SpreadStategyExchange,
}

/*----- */
// Nested structs required by Spread Strategy
/*----- */
#[derive(Debug, Default)]
pub struct SpreadStategyExchange {
    pub best_bid: Level,
    pub best_ask: Level,
    pub mid_price: Option<f64>,
    pub bps_offset_buy: VecDequeTime,
    pub bps_offset_sell: VecDequeTime,
    pub historical_spreads: VecDequeTime,
    pub current_spread: f64,
    pub fees: SpreadStrategyFees,
}

impl SpreadStategyExchange {
    pub fn new(withdrawl_fee: f64) -> Self {
        Self {
            best_bid: Level::new(0.0, 0.0),
            best_ask: Level::new(0.0, 0.0),
            mid_price: None,
            bps_offset_buy: VecDequeTime::new(3),
            bps_offset_sell: VecDequeTime::new(3),
            historical_spreads: VecDequeTime::new(3),
            current_spread: 0.0,
            fees: SpreadStrategyFees {
                market_risk: 0.0005,
                covergence_risk: 0.0005,
                withdrawl_fees: WithdrawalFee(withdrawl_fee),
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct SpreadDeque {
    pub data: VecDeque<f64>,
    pub capacity: usize,
}

#[derive(Debug, Default)]
pub struct VecDequeTime {
    pub data: VecDeque<f64>,
    pub tail_date: DateTime<Utc>,
    pub time_period: i64,
}

impl VecDequeTime {
    pub fn new(time_period: i64) -> Self {
        Self {
            data: VecDeque::new(),
            tail_date: Utc::now(),
            time_period,
        }
    }

    pub fn push(&mut self, current_date: DateTime<Utc>, value: f64) {
        if current_date < self.tail_date + Duration::hours(self.time_period) {
            self.data.push_front(value)
        } else {
            self.data.pop_back();
            self.data.push_front(value);
            self.tail_date = current_date;
        }
    }
}

impl SpreadDeque {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::<f64>::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.data.len() < self.capacity {
            self.data.push_front(value)
        } else {
            self.data.pop_back();
            self.data.push_front(value)
        }
    }
}

#[derive(Debug, Default)]
pub struct SpreadStrategyFees {
    pub market_risk: f64,
    pub covergence_risk: f64,
    pub withdrawl_fees: WithdrawalFee,
}

#[derive(Debug, Default)]
pub struct WithdrawalFee(pub f64);

impl WithdrawalFee {
    pub fn convert_bps(&self, current_price: f64) -> f64 {
        self.0 / current_price
    }
}

impl Default for SpreadStategy {
    fn default() -> Self {
        Self::new()
    }
}

/*----- */
// Impl SignalGenerator for SpreadStategy
/*----- */
impl SpreadStategy {
    pub fn new() -> Self {
        Self {
            liquid_exchange: SpreadStategyExchange::new(0.0021971),
            illiquid_exchange: SpreadStategyExchange::new(0.0021914),
        }
    }

    pub fn generate_signal_map(
        current_spread: f64,
        _current_fee: f64,
    ) -> HashMap<Decision, SignalStrength> {
        let mut signals = HashMap::with_capacity(4);

        if current_spread * -1.0 > 0.0 {
            signals.insert(Decision::Long, SignalStrength(5.0));
        } else {
            signals.insert(Decision::CloseLong, SignalStrength(1.0));
        }

        signals
    }

    pub fn update_state(&mut self, market_event: &MarketEvent<DataKind>) {
        match &market_event.event_data {
            // Process book data
            DataKind::OrderBook(book_data) => match market_event.exchange {
                ExchangeId::BinanceSpot => {
                    // Update midprice
                    let mid_price = book_data.midprice();
                    if self.liquid_exchange.mid_price != mid_price {
                        self.liquid_exchange.mid_price = mid_price
                    }

                    // Update best bid
                    if self.liquid_exchange.best_bid != book_data.bids[0] {
                        self.liquid_exchange.best_bid = book_data.bids[0];
                        // Since best bid changed update spread
                        if self.liquid_exchange.best_ask.price != 0.0 {
                            self.liquid_exchange.current_spread =
                                self.illiquid_exchange.best_bid.price
                                    / self.liquid_exchange.best_ask.price
                                    - 1.0;
                            self.liquid_exchange.historical_spreads.push(
                                market_event.exchange_time,
                                self.liquid_exchange.current_spread,
                            );

                            // println!(
                            //     "bin spread: {} || bin ask: {} || polo bid: {}",
                            //     self.liquid_exchange.current_spread,
                            //     self.liquid_exchange.best_ask.price,
                            //     self.illiquid_exchange.best_bid.price
                            // )
                        }
                    }

                    // Update best ask
                    if self.liquid_exchange.best_ask != book_data.asks[0] {
                        self.liquid_exchange.best_ask = book_data.asks[0];
                        // Since best ask changed update spread
                        if self.liquid_exchange.best_ask.price != 0.0 {
                            self.liquid_exchange.current_spread =
                                self.illiquid_exchange.best_bid.price
                                    / self.liquid_exchange.best_ask.price
                                    - 1.0;
                            self.liquid_exchange.historical_spreads.push(
                                market_event.exchange_time,
                                self.liquid_exchange.current_spread,
                            );

                            // println!(
                            //     "bin spread: {} || bin ask: {:?} || polo bid: {:?}",
                            //     self.liquid_exchange.current_spread,
                            //     self.liquid_exchange.best_ask,
                            //     self.illiquid_exchange.best_bid
                            // )
                        }
                    }
                }
                ExchangeId::PoloniexSpot => {
                    // Update mid price
                    let mid_price = book_data.midprice();
                    if self.illiquid_exchange.mid_price != mid_price {
                        self.illiquid_exchange.mid_price = mid_price
                    }

                    // Update best bid
                    if self.illiquid_exchange.best_bid != book_data.bids[0] {
                        self.illiquid_exchange.best_bid = book_data.bids[0];
                        // Since best bid changed update spread
                        if self.illiquid_exchange.best_ask.price != 0.0 {
                            self.illiquid_exchange.current_spread =
                                self.liquid_exchange.best_bid.price
                                    / self.illiquid_exchange.best_ask.price
                                    - 1.0;
                            self.illiquid_exchange.historical_spreads.push(
                                market_event.exchange_time,
                                self.illiquid_exchange.current_spread,
                            );
                            // println!(
                            //     "polo spread: {} || polo ask: {:?} || bin bid: {:?}",
                            //     self.illiquid_exchange.current_spread,
                            //     self.illiquid_exchange.best_ask,
                            //     self.liquid_exchange.best_bid
                            // );
                        }
                    }

                    // Update best ask
                    if self.illiquid_exchange.best_ask != book_data.asks[0] {
                        self.illiquid_exchange.best_ask = book_data.asks[0];
                        // Since best ask changed update spread
                        if self.illiquid_exchange.best_ask.price != 0.0 {
                            self.illiquid_exchange.current_spread =
                                self.liquid_exchange.best_bid.price
                                    / self.illiquid_exchange.best_ask.price
                                    - 1.0;
                            self.illiquid_exchange.historical_spreads.push(
                                market_event.exchange_time,
                                self.illiquid_exchange.current_spread,
                            );
                            // println!(
                            //     "polo spread: {} || polo ask: {:?} || bin bid: {:?}",
                            //     self.illiquid_exchange.current_spread,
                            //     self.illiquid_exchange.best_ask,
                            //     self.liquid_exchange.best_bid
                            // );
                        }
                    }
                }
            },
            // Process trade data
            DataKind::Trade(event_trade) => match market_event.exchange {
                ExchangeId::BinanceSpot => {
                    if let Some(mid_price) = self.liquid_exchange.mid_price {
                        // Calculate bps offset
                        let bps_offset = (event_trade.trade.price - mid_price) / mid_price;

                        // Update historical bps offset
                        match event_trade.is_buy {
                            true => {
                                self.liquid_exchange
                                    .bps_offset_buy
                                    .push(market_event.exchange_time, bps_offset);
                            }
                            false => {
                                self.liquid_exchange
                                    .bps_offset_sell
                                    .push(market_event.exchange_time, bps_offset);
                            }
                        }
                    }
                }
                ExchangeId::PoloniexSpot => {
                    // Calcuate bps offset
                    if let Some(mid_price) = self.illiquid_exchange.mid_price {
                        let bps_offset = (event_trade.trade.price - mid_price) / mid_price;

                        // Update historical bps offset
                        match event_trade.is_buy {
                            true => {
                                self.illiquid_exchange
                                    .bps_offset_buy
                                    .push(market_event.exchange_time, bps_offset);
                            }
                            false => {
                                self.illiquid_exchange
                                    .bps_offset_sell
                                    .push(market_event.exchange_time, bps_offset);
                            }
                        }
                    }
                }
            },
        }
    }
}

impl SignalGenerator for SpreadStategy {
    fn generate_signal(&mut self, market_event: &MarketEvent<DataKind>) -> Option<Signal> {
        let old_exchange_one_bid = self.liquid_exchange.best_bid.price;
        let old_exchange_one_ask = self.liquid_exchange.best_ask.price;
        let old_exchange_two_bid = self.illiquid_exchange.best_bid.price;
        let old_exchange_two_ask = self.illiquid_exchange.best_ask.price;

        self.update_state(market_event);

        let new_exchange_one_bid = self.liquid_exchange.best_bid.price;
        let new_exchange_one_ask = self.liquid_exchange.best_ask.price;
        let new_exchange_two_bid = self.illiquid_exchange.best_bid.price;
        let new_exchange_two_ask = self.illiquid_exchange.best_ask.price;

        // Generate signal for if exchange one. Note the exchange spreads are
        // calculated by finding the difference between exchange one's ask and
        // exchange two's bids
        if old_exchange_one_ask != new_exchange_one_ask
            || old_exchange_two_bid != new_exchange_two_bid
        {
            let fee_hurdle = self.liquid_exchange.fees.covergence_risk
                + self.liquid_exchange.fees.market_risk
                + self
                    .liquid_exchange
                    .fees
                    .withdrawl_fees
                    .convert_bps(self.liquid_exchange.best_ask.price);

            let signals =
                SpreadStategy::generate_signal_map(self.liquid_exchange.current_spread, fee_hurdle);

            // println!("bin signal: {:?}", signals);

            if signals.is_empty() {
                return None;
            }

            Some(Signal {
                time: Utc::now(),
                exchange: market_event.exchange,
                instrument: market_event.instrument.clone(),
                signals,
                market_meta: MarketMeta {
                    close: self.liquid_exchange.best_ask.price,
                    time: Utc::now(),
                },
            })
        }
        // Generate exchange two signal
        else if old_exchange_two_ask != new_exchange_two_ask
            || old_exchange_one_bid != new_exchange_one_bid
        {
            let fee_hurdle = self.illiquid_exchange.fees.covergence_risk
                + self.illiquid_exchange.fees.market_risk
                + self
                    .illiquid_exchange
                    .fees
                    .withdrawl_fees
                    .convert_bps(self.illiquid_exchange.best_ask.price);

            let signals = SpreadStategy::generate_signal_map(
                self.illiquid_exchange.current_spread,
                fee_hurdle,
            );

            // println!("polo signal: {:?}", signals);

            if signals.is_empty() {
                return None;
            }

            Some(Signal {
                time: Utc::now(),
                exchange: market_event.exchange,
                instrument: market_event.instrument.clone(),
                signals,
                market_meta: MarketMeta {
                    close: self.illiquid_exchange.best_ask.price,
                    time: Utc::now(),
                },
            })
        }
        // Dont generate signal if prices have not changed
        else {
            return None;
        }
    }
}

/*----- */
// Todo
/*----- */
// dynamic way to calc optimal bp offset
// heuristic on how long the arb will last
// seems poloneix trade streams are aggregated but binance is not
