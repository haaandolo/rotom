use crate::{
    exchange::Identifier,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trade},
    shared::subscription_models::Subscription,
};

use super::OkxSpotPublicData;

#[derive(Debug)]
pub struct OkxChannel(pub &'static str);

impl OkxChannel {
    pub const TRADES: Self = Self("trades");
    pub const ORDERBOOKSNAPSHOT: Self = Self("books5");
}

impl AsRef<str> for OkxChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<OkxChannel> for Subscription<OkxSpotPublicData, OrderBookSnapshot> {
    fn id(&self) -> OkxChannel {
        OkxChannel::ORDERBOOKSNAPSHOT
    }
}

impl Identifier<OkxChannel> for Subscription<OkxSpotPublicData, Trade> {
    fn id(&self) -> OkxChannel {
        OkxChannel::TRADES
    }
}
