use crate::{
    exchange::Identifier,
    model::{event_book_snapshot::OrderBookSnapshot, event_trade::Trades},
    shared::subscription_models::Subscription,
};

use super::CoinExSpotPublicData;

#[derive(Debug, Clone, Copy)]
pub struct CoinExChannel(pub &'static str);

impl CoinExChannel {
    pub const TRADES: Self = Self("deals.subscribe");
    pub const ORDERBOOKSNAPSHOT: Self = Self("depth.subscribe");
}

impl AsRef<str> for CoinExChannel {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Identifier<CoinExChannel> for Subscription<CoinExSpotPublicData, OrderBookSnapshot> {
    fn id(&self) -> CoinExChannel {
        CoinExChannel::ORDERBOOKSNAPSHOT
    }
}

impl Identifier<CoinExChannel> for Subscription<CoinExSpotPublicData, Trades> {
    fn id(&self) -> CoinExChannel {
        CoinExChannel::TRADES
    }
}
