use crate::{
    exchange::Identifier,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    shared::subscription_models::Subscription,
};

use super::ExmoSpotPublicData;

#[derive(Debug)]
pub struct ExmoChannel(pub &'static str);

impl ExmoChannel {
    pub const TRADES: Self = Self("spot/trades:");
    pub const ORDERBOOKSNAPSHOT: Self = Self("spot/order_book_snapshots:");
}

impl AsRef<str> for ExmoChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<ExmoChannel> for Subscription<ExmoSpotPublicData, OrderBookSnapshot> {
    fn id(&self) -> ExmoChannel {
        ExmoChannel::ORDERBOOKSNAPSHOT
    }
}

impl Identifier<ExmoChannel> for Subscription<ExmoSpotPublicData, Trades> {
    fn id(&self) -> ExmoChannel {
        ExmoChannel::TRADES
    }
}
