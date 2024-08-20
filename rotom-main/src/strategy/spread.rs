use std::collections::HashMap;

use chrono::Utc;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent}, exchange, shared::subscription_models::ExchangeId
};

use crate::data::MarketMeta;

use super::{Decision, Signal, SignalGenerator, SignalStrength};
/*----- */
// Spread strategy
/*----- */
#[derive(Debug, Default)]
pub struct SpreadStategy {
    // Mid price
    pub exchange_one_mid_price: Option<f64>,
    pub exchange_two_mid_price: Option<f64>,
    // Offsets
    pub exchange_one_bps_offsets: Vec<f64>, // TODO: rm vec
    pub exchange_two_bps_offsets: Vec<f64>, // TODO: rm vec
    // BB
    pub exchange_one_bb: f64,
    pub exchange_two_bb: f64,
    // BA
    pub exchange_one_ba: f64,
    pub exchange_two_ba: f64,
    // Spread
    pub exchange_one_spread: f64,
    pub exchange_two_spread: f64,
}

// impl default for spreadstategy {
//     fn default() -> self {
//         self {
//         }
//     }
// }

impl SignalGenerator for SpreadStategy {
    fn generate_signal(&mut self, market: &MarketEvent<DataKind>) -> Option<Signal> {
        match &market.event_data {
            // Process book data
            DataKind::OrderBook(book_data) => match market.exchange {
                ExchangeId::BinanceSpot => {
                    self.exchange_one_mid_price = book_data.midprice();

                    if self.exchange_one_bb != book_data.bids[0].price {
                        self.exchange_one_bb = book_data.bids[0].price;
                    }

                    if self.exchange_one_ba != book_data.asks[0].price {
                        self.exchange_one_ba = book_data.asks[0].price;
                    }

                }
                ExchangeId::PoloniexSpot => {
                    self.exchange_two_mid_price = book_data.midprice();

                    if self.exchange_two_bb != book_data.bids[0].price {
                        self.exchange_two_bb = book_data.bids[0].price;
                    }

                    if self.exchange_two_ba != book_data.asks[0].price {
                        self.exchange_two_ba = book_data.asks[0].price;
                    }
                }
            },
            // Process trade data
            DataKind::Trade(event_trade) => match market.exchange {
                ExchangeId::BinanceSpot => {
                    if let Some(mid_price) = self.exchange_one_mid_price {
                        let bps_offset = (event_trade.trade.price - mid_price) / mid_price;
                        self.exchange_one_bps_offsets.push(bps_offset);
                    }
                }
                ExchangeId::PoloniexSpot => {
                    if let Some(mid_price) = self.exchange_two_mid_price {
                        let bps_offset = (event_trade.trade.price - mid_price) / mid_price;
                        self.exchange_two_bps_offsets.push(bps_offset);
                    }
                }

            }
        }

        // Exchange one spread
        let exchange_one_spread = self.exchange_one_ba / self.exchange_two_bb;
        if self.exchange_one_spread != exchange_one_spread {
            self.exchange_one_spread = exchange_one_spread;
            println!("Bin spread: {}", exchange_one_spread)
        }

        // Exchange two spread
        let exchange_two_spread = self.exchange_two_ba / self.exchange_one_bb;
        if self.exchange_two_spread != exchange_two_spread {
            self.exchange_two_spread = exchange_two_spread;
            println!("Polo spread: {}", exchange_two_spread)
        }

        let signals = SpreadStategy::generate_signal_map(89.1);

        if signals.is_empty() {
            return None;
        }

        Some(Signal {
            time: Utc::now(),
            exchange: market.exchange,
            instrument: market.instrument.clone(),
            signals,
            market_meta: MarketMeta {
                close: 90.0,
                time: Utc::now(),
            },
        })
    }
}

impl SpreadStategy {
    pub fn generate_signal_map(bid_spread: f64) -> HashMap<Decision, SignalStrength> {
        let mut signals = HashMap::with_capacity(4);
        if bid_spread > 0.001 {
            signals.insert(Decision::Long, SignalStrength(1.0));
        }

        if bid_spread < 0.0001 {
            signals.insert(Decision::Short, SignalStrength(-1.0));
        }

        signals
    }
}

/*----- */
// Todo
/*----- */
// dynamic way to calc optimal bp offset
// heuristic on how long the arb will last
// seems poloneix trade streams are aggregated but binance is not
