use super::BinanceSpot;
use crate::data::{
    exchange::Identifier,
    event_models::{event_book::OrderBookL2, event_trade::Trades}, shared::subscription_models::Subscription
};

#[derive(Debug)]
pub struct BinanceChannel(pub &'static str);

impl BinanceChannel {
    pub const TRADES: Self = Self("@trade");
    pub const ORDER_BOOK_L1: Self = Self("@bookTicker");
    pub const ORDER_BOOK_L2: Self = Self("@depth@100ms");
    pub const LIQUIDATIONS: Self = Self("@forceOrder");
}

impl AsRef<str> for BinanceChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<BinanceChannel> for Subscription<BinanceSpot, OrderBookL2> {
    fn id(&self) -> BinanceChannel {
        BinanceChannel::ORDER_BOOK_L2
    }
}
impl Identifier<BinanceChannel> for Subscription<BinanceSpot, Trades> {
    fn id(&self) -> BinanceChannel {
        BinanceChannel::TRADES
    }
}
