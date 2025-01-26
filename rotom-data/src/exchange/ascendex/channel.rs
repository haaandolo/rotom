use crate::{
    exchange::Identifier,
    model::{event_book::OrderBookL2, event_trade::Trade},
    shared::subscription_models::Subscription,
};

use super::AscendExSpotPublicData;

#[derive(Debug)]
pub struct AscendExChannel(pub &'static str);

impl AscendExChannel {
    pub const TRADES: Self = Self("trades:");
    pub const ORDER_BOOK_L2: Self = Self("depth:");
}

impl AsRef<str> for AscendExChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<AscendExChannel> for Subscription<AscendExSpotPublicData, OrderBookL2> {
    fn id(&self) -> AscendExChannel {
        AscendExChannel::ORDER_BOOK_L2
    }
}

impl Identifier<AscendExChannel> for Subscription<AscendExSpotPublicData, Trade> {
    fn id(&self) -> AscendExChannel {
        AscendExChannel::TRADES
    }
}
