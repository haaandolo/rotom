use std::collections::{HashMap, VecDeque};

use chrono::Utc;
use rotom_data::{
    assets::level::Level,
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
};

use super::{Decision, Signal, SignalGenerator, SignalStrength};
use crate::data::MarketMeta;

/*----- */
// Spread strategy
/*----- */
#[derive(Debug)]
pub struct SpreadStategy {
    pub exchange_one: SpreadStategyExchange,
    pub exchange_two: SpreadStategyExchange,
}

impl SpreadStategy {
    pub fn new() -> Self {
        Self {
            exchange_one: SpreadStategyExchange::new(0.0021971),
            exchange_two: SpreadStategyExchange::new(0.0021914),
        }
    }

    pub fn generate_signal_map(
        current_spread: f64,
        current_fee: f64,
    ) -> HashMap<Decision, SignalStrength> {
        let mut signals = HashMap::with_capacity(4);

        if current_spread - 1.0 > current_fee {
            signals.insert(Decision::Long, SignalStrength(1.0));
        }

        signals
    }
}

/*----- */
// Nested structs required by Spread Strategy
/*----- */
#[derive(Debug, Default)]
pub struct SpreadStategyExchange {
    pub best_bid: Level,
    pub best_ask: Level,
    pub mid_price: Option<f64>,
    pub bps_offset_buy: SpreadDeque,
    pub bps_offset_sell: SpreadDeque,
    pub historical_spreads: SpreadDeque,
    pub current_spread: f64,
    pub fees: SpreadStrategyFees,
}

impl SpreadStategyExchange {
    pub fn new(withdrawl_fee: f64) -> Self {
        Self {
            best_bid: Level::new(0.0, 0.0),
            best_ask: Level::new(0.0, 0.0),
            mid_price: None,
            bps_offset_buy: SpreadDeque::new(5000),
            bps_offset_sell: SpreadDeque::new(5000),
            historical_spreads: SpreadDeque::new(5000),
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

impl SpreadDeque {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::<f64>::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.capacity < self.data.len() {
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

/*----- */
// Impl SignalGenerator for SpreadStategy
/*----- */
impl SignalGenerator for SpreadStategy {
    fn generate_signal(&mut self, market_event: &MarketEvent<DataKind>) -> Option<Signal> {
        let old_exchange_one_bid = self.exchange_one.best_bid.price;
        let old_exchange_one_ask = self.exchange_one.best_ask.price;
        let old_exchange_two_bid = self.exchange_two.best_bid.price;
        let old_exchange_two_ask = self.exchange_two.best_ask.price;

        match &market_event.event_data {
            // Process book data
            DataKind::OrderBook(book_data) => match market_event.exchange {
                ExchangeId::BinanceSpot => {
                    // Update midprice
                    self.exchange_one.mid_price = book_data.midprice();

                    // Update best bid
                    if self.exchange_one.best_bid != book_data.bids[0] {
                        self.exchange_one.best_bid = book_data.bids[0];
                    }

                    // Update best ask
                    if self.exchange_one.best_ask != book_data.asks[0] {
                        self.exchange_one.best_ask = book_data.asks[0];
                    }

                    // Calc spread
                    let exchange_one_spread =
                        self.exchange_one.best_ask.price / self.exchange_two.best_bid.price;

                    // Update historical spread
                    if self.exchange_one.current_spread != exchange_one_spread {
                        self.exchange_one.current_spread = exchange_one_spread;

                        self.exchange_one
                            .historical_spreads
                            .push(exchange_one_spread);
                        // println!("Bin spread: {}", exchange_one_spread)
                    }
                }
                ExchangeId::PoloniexSpot => {
                    // Update mid price
                    self.exchange_two.mid_price = book_data.midprice();

                    // Update best bid
                    if self.exchange_two.best_bid != book_data.bids[0] {
                        self.exchange_two.best_bid = book_data.bids[0];
                    }

                    // Update best ask
                    if self.exchange_two.best_ask != book_data.asks[0] {
                        self.exchange_two.best_ask = book_data.asks[0];
                    }

                    // Calc spread
                    let exchange_two_spread =
                        self.exchange_two.best_ask.price / self.exchange_one.best_bid.price;

                    // Update historical spread
                    if self.exchange_two.current_spread != exchange_two_spread {
                        self.exchange_two.current_spread = exchange_two_spread;
                        self.exchange_two
                            .historical_spreads
                            .push(exchange_two_spread);
                        // println!("Polo spread: {}", exchange_two_spread)
                    }
                }
            },
            // Process trade data
            DataKind::Trade(event_trade) => match market_event.exchange {
                ExchangeId::BinanceSpot => {
                    if let Some(mid_price) = self.exchange_one.mid_price {
                        // Calculate bps offset
                        let bps_offset = (event_trade.trade.price - mid_price) / mid_price;

                        // Update historical bps offset
                        match event_trade.is_buy {
                            true => {
                                self.exchange_one.bps_offset_buy.push(bps_offset);
                            }
                            false => {
                                self.exchange_one.bps_offset_sell.push(bps_offset);
                            }
                        }
                    }
                }
                ExchangeId::PoloniexSpot => {
                    // Calcuate bps offset
                    if let Some(mid_price) = self.exchange_two.mid_price {
                        let bps_offset = (event_trade.trade.price - mid_price) / mid_price;

                        // Update historical bps offset
                        match event_trade.is_buy {
                            true => {
                                self.exchange_two.bps_offset_buy.push(bps_offset);
                            }
                            false => {
                                self.exchange_two.bps_offset_sell.push(bps_offset);
                            }
                        }
                    }
                }
            },
        }

        let new_exchange_one_bid = self.exchange_one.best_bid.price;
        let new_exchange_one_ask = self.exchange_one.best_ask.price;
        let new_exchange_two_bid = self.exchange_two.best_bid.price;
        let new_exchange_two_ask = self.exchange_two.best_ask.price;

        // Generate signal for if exchange one. Note the exchange spreads are
        // calculated by finding the difference between exchange one's ask and
        // exchange two's bids
        if old_exchange_one_ask != new_exchange_one_ask
            || old_exchange_two_bid != new_exchange_two_bid
        {
            let fee_hurdle = self.exchange_one.fees.covergence_risk
                + self.exchange_one.fees.market_risk
                + self
                    .exchange_one
                    .fees
                    .withdrawl_fees
                    .convert_bps(self.exchange_one.best_ask.price);

            let signals =
                SpreadStategy::generate_signal_map(self.exchange_one.current_spread, fee_hurdle);

            // println!("bin signal: {:?}", signals);

            if signals.is_empty() {
                return None;
            }

            return Some(Signal {
                time: Utc::now(),
                exchange: market_event.exchange,
                instrument: market_event.instrument.clone(),
                signals,
                market_meta: MarketMeta {
                    close: self.exchange_one.best_ask.price,
                    time: Utc::now(),
                },
            });
        }
        // Generate exchange two signal
        else if old_exchange_two_ask != new_exchange_two_ask
            || old_exchange_one_bid != new_exchange_one_bid
        {
            let fee_hurdle = self.exchange_two.fees.covergence_risk
                + self.exchange_two.fees.market_risk
                + self
                    .exchange_two
                    .fees
                    .withdrawl_fees
                    .convert_bps(self.exchange_two.best_ask.price);

            let signals =
                SpreadStategy::generate_signal_map(self.exchange_two.current_spread, fee_hurdle);

            // println!("polo signal: {:?}", signals);

            if signals.is_empty() {
                return None;
            }

            return Some(Signal {
                time: Utc::now(),
                exchange: market_event.exchange,
                instrument: market_event.instrument.clone(),
                signals,
                market_meta: MarketMeta {
                    close: self.exchange_two.best_ask.price,
                    time: Utc::now(),
                },
            });
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
