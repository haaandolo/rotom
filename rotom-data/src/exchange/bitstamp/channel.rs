use crate::{
    exchange::Identifier,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    shared::subscription_models::Subscription,
};

use super::BitstampSpotPublicData;

#[derive(Debug)]
pub struct BitstampChannel(pub &'static str);

impl BitstampChannel {
    pub const TRADES: Self = Self("live_orders_");
    pub const ORDERBOOKSNAPSHOT: Self = Self("order_book_");
}

impl AsRef<str> for BitstampChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<BitstampChannel> for Subscription<BitstampSpotPublicData, OrderBookSnapshot> {
    fn id(&self) -> BitstampChannel {
        BitstampChannel::ORDERBOOKSNAPSHOT
    }
}

impl Identifier<BitstampChannel> for Subscription<BitstampSpotPublicData, Trades> {
    fn id(&self) -> BitstampChannel {
        BitstampChannel::TRADES
    }
}
