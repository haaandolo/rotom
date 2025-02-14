use super::BinanceSpotPublicData;
use crate::{exchange::Identifier, shared::subscription_models::Subscription};

#[derive(Debug)]
pub struct BinanceMarket(pub String);

impl<StreamKind> Identifier<BinanceMarket> for Subscription<BinanceSpotPublicData, StreamKind> {
    fn id(&self) -> BinanceMarket {
        BinanceMarket(format!("{}{}", self.instrument.base, self.instrument.quote).to_uppercase())
    }
}

impl AsRef<str> for BinanceMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
