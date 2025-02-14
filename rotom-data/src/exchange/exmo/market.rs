use crate::{exchange::Identifier, shared::subscription_models::Subscription};

use super::ExmoSpotPublicData;

#[derive(Debug)]
pub struct ExmoMarket(pub String);

impl<StreamKind> Identifier<ExmoMarket> for Subscription<ExmoSpotPublicData, StreamKind> {
    fn id(&self) -> ExmoMarket {
        ExmoMarket(format!("{}_{}", self.instrument.base, self.instrument.quote).to_uppercase())
    }
}

impl AsRef<str> for ExmoMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
