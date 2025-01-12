use super::PoloniexSpotPublicData;
use crate::{
    exchange::Identifier,
    model::{event_book::OrderBookL2, event_trade::Trades},
    shared::subscription_models::Subscription,
};

#[derive(Debug)]
pub struct PoloniexChannel(pub &'static str);

impl PoloniexChannel {
    pub const TRADES: Self = Self("trades");
    pub const ORDER_BOOK_L2: Self = Self("book_lv2");
}

impl AsRef<str> for PoloniexChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<PoloniexChannel> for Subscription<PoloniexSpotPublicData, OrderBookL2> {
    fn id(&self) -> PoloniexChannel {
        PoloniexChannel::ORDER_BOOK_L2
    }
}

impl Identifier<PoloniexChannel> for Subscription<PoloniexSpotPublicData, Trades> {
    fn id(&self) -> PoloniexChannel {
        PoloniexChannel::TRADES
    }
}
