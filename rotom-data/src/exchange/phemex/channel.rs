use crate::{
    exchange::Identifier,
    model::{event_book::OrderBookL2, event_trade::Trade},
    shared::subscription_models::Subscription,
};

use super::PhemexSpotPublicData;

#[derive(Debug)]
pub struct PhemexChannel(pub &'static str);

impl PhemexChannel {
    pub const TRADES: Self = Self("@trade");
    pub const ORDER_BOOK_L2: Self = Self("orderbook.subscribe");
}

impl AsRef<str> for PhemexChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<PhemexChannel> for Subscription<PhemexSpotPublicData, OrderBookL2> {
    fn id(&self) -> PhemexChannel {
        PhemexChannel::ORDER_BOOK_L2
    }
}

// impl Identifier<PhemexChannel> for Subscription<PhemexSpotPublicData, Trade> {
//     fn id(&self) -> PhemexChannel {
//         PhemexChannel::TRADES
//     }
// }
