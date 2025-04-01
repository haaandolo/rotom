use crate::{
    exchange::Identifier,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trade},
    shared::subscription_models::Subscription,
};

use super::KuCoinSpotPublicData;

#[derive(Debug, Clone, Copy)]
pub struct KuCoinChannel(pub &'static str);

impl KuCoinChannel {
    pub const TRADES: Self = Self("/market/match:");
    pub const ORDERBOOKSNAPSHOT: Self = Self("/spotMarket/level2Depth50:");
}

impl AsRef<str> for KuCoinChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<KuCoinChannel> for Subscription<KuCoinSpotPublicData, OrderBookSnapshot> {
    fn id(&self) -> KuCoinChannel {
        KuCoinChannel::ORDERBOOKSNAPSHOT
    }
}

impl Identifier<KuCoinChannel> for Subscription<KuCoinSpotPublicData, Trade> {
    fn id(&self) -> KuCoinChannel {
        KuCoinChannel::TRADES
    }
}
