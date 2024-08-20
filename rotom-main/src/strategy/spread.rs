use std::collections::HashMap;

use chrono::Utc;
use rotom_data::{
    assets::level::Level,
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
};

use crate::data::MarketMeta;

use super::{Decision, Signal, SignalGenerator, SignalStrength};
/*----- */
// Spread strategy
/*----- */
#[derive(Debug, Default)]
pub struct SpreadStategy {
    pub exchange_one_mid_price: Option<f64>,
    pub exchange_two_mid_price: Option<f64>,
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

                }
                ExchangeId::PoloniexSpot => {
                    self.exchange_two_mid_price = book_data.midprice();
                }
            },
            // Process trade data
            DataKind::Trade(event_trade) => match market.exchange {
                ExchangeId::BinanceSpot => {
                    // println!("{:?}", event_trade);
                    if let Some(mid_price) = self.exchange_one_mid_price {
                        let bps_offset = (event_trade.trade.price - mid_price) / mid_price;
                        println!("Bin: {:#?}", bps_offset);
                    }
                }
                ExchangeId::PoloniexSpot => {
                    // println!("{:?}", event_trade);
                    if let Some(mid_price) = self.exchange_two_mid_price {
                        let bps_offset = (event_trade.trade.price - mid_price) / mid_price;
                        println!("Polo: {:#?}", bps_offset);
                    }
                }

            }
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
