use crate::{
    exchange::Identifier,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    shared::subscription_models::Subscription,
};

use super::HtxSpotPublicData;

#[derive(Debug)]
pub struct HtxChannel(pub &'static str);

impl HtxChannel {
    pub const TRADES: Self = Self("trade.detail");
    pub const ORDERBOOKSNAPSHOT: Self = Self("mbp.refresh.5");
}

impl AsRef<str> for HtxChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<HtxChannel> for Subscription<HtxSpotPublicData, OrderBookSnapshot> {
    fn id(&self) -> HtxChannel {
        HtxChannel::ORDERBOOKSNAPSHOT
    }
}

impl Identifier<HtxChannel> for Subscription<HtxSpotPublicData, Trades> {
    fn id(&self) -> HtxChannel {
        HtxChannel::TRADES
    }
}
