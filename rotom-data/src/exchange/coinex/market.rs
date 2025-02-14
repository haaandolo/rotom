use crate::{exchange::Identifier, shared::subscription_models::Subscription};

use super::CoinExSpotPublicData;

#[derive(Debug)]
pub struct CoinExMarket(pub String);

impl<StreamKind> Identifier<CoinExMarket> for Subscription<CoinExSpotPublicData, StreamKind> {
    fn id(&self) -> CoinExMarket {
        CoinExMarket(format!("{}{}", self.instrument.base, self.instrument.quote).to_uppercase())
    }
}

impl AsRef<str> for CoinExMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
