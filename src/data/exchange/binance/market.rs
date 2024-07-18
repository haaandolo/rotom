use crate::data::{exchange::Identifier, models::subs::Subscription};

use super::BinanceSpot;

pub struct BinanceMarket(pub String);

impl<StreamKind> Identifier<BinanceMarket> for Subscription<BinanceSpot, StreamKind> {
    fn id(&self) -> BinanceMarket {
        BinanceMarket(format!("{}{}", self.instrument.base, self.instrument.quote).to_uppercase())
    }
}

impl AsRef<str> for BinanceMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}