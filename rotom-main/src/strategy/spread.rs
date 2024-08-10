use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
};

use super::{Signal, SignalGenerator};

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
    fn generate_signal(&mut self, market: &MarketEvent<DataKind>) -> Option<Signal> {
        let book_data = match &market.event_data {
            DataKind::OrderBook(book) => book,
            _ => return None,
        };

        match market.exchange {
            ExchangeId::BinanceSpot => {
                self.exchange_one_bid = book_data.bids[0].price;
                self.exchange_one_ask = book_data.asks[0].price;
            }
            ExchangeId::PoloniexSpot => {
                self.exchange_two_bid = book_data.bids[0].price;
                self.exchange_two_ask = book_data.asks[0].price;
            }
        }

        let bid_spread = (self.exchange_one_bid - self.exchange_two_bid).abs();
        let ask_spread = (self.exchange_one_ask - self.exchange_two_ask).abs();

        println!("Bid spread: {}", bid_spread);
        println!("Ask spread: {}", ask_spread);

        None
    }
}
