use crate::data::{exchange::Identifier, models::{book::OrderBookL2, subs::Subscription, trade::Trades}};

use super::BinanceSpot;

pub struct BinanceChannel(pub &'static str);

impl BinanceChannel {
    pub const SPOT_WS_URL: Self = Self("wss://stream.binance.com:9443/ws");
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