use super::BinanceSpotPublicData;
use crate::{
    exchange::Identifier,
    model::{
        event_book::OrderBookL2,
        event_trade::{AggTrades, Trade},
    },
    shared::subscription_models::Subscription,
};

#[derive(Debug)]
pub struct BinanceChannel(pub &'static str);

impl BinanceChannel {
    pub const TRADES: Self = Self("@trade");
    pub const AGGREGATED_TRADES: Self = Self("@aggTrade");
    pub const ORDER_BOOK_L1: Self = Self("@bookTicker");
    pub const ORDER_BOOK_L2: Self = Self("@depth@100ms");
    pub const LIQUIDATIONS: Self = Self("@forceOrder");
}

impl AsRef<str> for BinanceChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<BinanceChannel> for Subscription<BinanceSpotPublicData, OrderBookL2> {
    fn id(&self) -> BinanceChannel {
        BinanceChannel::ORDER_BOOK_L2
    }
}

impl Identifier<BinanceChannel> for Subscription<BinanceSpotPublicData, Trade> {
    fn id(&self) -> BinanceChannel {
        BinanceChannel::TRADES
    }
}

impl Identifier<BinanceChannel> for Subscription<BinanceSpotPublicData, AggTrades> {
    fn id(&self) -> BinanceChannel {
        BinanceChannel::AGGREGATED_TRADES
    }
}
