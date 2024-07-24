use crate::data::{
    exchange::Identifier,
    models::{book::OrderBookL2, subs::Subscription, trade::Trades},
};
use super::PoloniexSpot;

pub struct PoloniexChannel(pub &'static str);

impl PoloniexChannel {
    pub const SPOT_WS_URL: Self = Self("wss://ws.poloniex.com/ws/public");
    pub const TRADES: Self = Self("trades");
    pub const ORDER_BOOK_L2: Self = Self("book_lv2");
}

impl AsRef<str> for PoloniexChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<PoloniexChannel> for Subscription<PoloniexSpot, OrderBookL2> {
    fn id(&self) -> PoloniexChannel {
        PoloniexChannel::ORDER_BOOK_L2
    }
}

impl Identifier<PoloniexChannel> for Subscription<PoloniexSpot, Trades> {
    fn id(&self) -> PoloniexChannel {
        PoloniexChannel::TRADES
    }
}
