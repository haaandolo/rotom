use crate::{exchange::Identifier, shared::subscription_models::Subscription};

use super::KuCoinSpotPublicData;

#[derive(Debug)]
pub struct KuCoinMarket(pub String);

impl<StreamKind> Identifier<KuCoinMarket> for Subscription<KuCoinSpotPublicData, StreamKind> {
    fn id(&self) -> KuCoinMarket {
        KuCoinMarket(format!("{}-{}", self.instrument.base, self.instrument.quote).to_uppercase())
    }
}

impl AsRef<str> for KuCoinMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
