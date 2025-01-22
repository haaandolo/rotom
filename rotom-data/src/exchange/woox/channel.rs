use crate::{
    exchange::Identifier,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    shared::subscription_models::Subscription,
};

use super::WooxSpotPublicData;

#[derive(Debug)]
pub struct WooxChannel(pub &'static str);

impl WooxChannel {
    pub const TRADES: Self = Self("@trade");
    pub const ORDERBOOKSNAPSHOT: Self = Self("@orderbook");
}

impl AsRef<str> for WooxChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<WooxChannel> for Subscription<WooxSpotPublicData, OrderBookSnapshot> {
    fn id(&self) -> WooxChannel {
        WooxChannel::ORDERBOOKSNAPSHOT
    }
}

impl Identifier<WooxChannel> for Subscription<WooxSpotPublicData, Trades> {
    fn id(&self) -> WooxChannel {
        WooxChannel::TRADES
    }
}
