use std::collections::HashMap;

use chrono::Utc;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
};

use crate::data::MarketMeta;

use super::{Decision, Signal, SignalGenerator, SignalStrength};
/*----- */
// Spread strategy
/*----- */
#[derive(Debug)]
pub struct SpreadStategy {
    exchange_one_bid: f64,
    exchange_one_ask: f64,
    exchange_two_bid: f64,
    exchange_two_ask: f64,
}

impl Default for SpreadStategy {
    fn default() -> Self {
        Self {
            exchange_one_bid: 0.0,
            exchange_one_ask: 0.0,
            exchange_two_bid: 0.0,
            exchange_two_ask: 0.0,
        }
    }
}

impl SignalGenerator for SpreadStategy {
    // Note: rn the exchange dont send data at the same time
    // make sure to implement some logic that takes in takes this into account
    fn generate_signal(&mut self, market: &MarketEvent<DataKind>) -> Option<Signal> {
        match &market.event_data {
            DataKind::OrderBook(book_data) => match market.exchange {
                ExchangeId::BinanceSpot => {
                    self.exchange_one_bid = book_data.bids[0].price;
                    self.exchange_one_ask = book_data.asks[0].price;
                }
                ExchangeId::PoloniexSpot => {
                    self.exchange_two_bid = book_data.bids[0].price;
                    self.exchange_two_ask = book_data.asks[0].price;
                }
            },
            // DataKind::Trade(trade) => {
            //     println!("Trade {:?}: {:?}", market.exchange, trade.trade.price)
            // }
            _ => return None,
        }
        let bid_spread = (self.exchange_one_bid - self.exchange_two_bid).abs();
        let signals = SpreadStategy::generate_signal_map(bid_spread);

        if signals.is_empty() {
            return None;
        }

        Some(Signal {
            time: Utc::now(),
            exchange: market.exchange,
            instrument: market.instrument.clone(),
            signals,
            market_meta: MarketMeta {
                close: self.exchange_one_bid,
                time: Utc::now()
            }
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
