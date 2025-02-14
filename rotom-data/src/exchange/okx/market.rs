use crate::{exchange::Identifier, shared::subscription_models::Subscription};

use super::OkxSpotPublicData;

#[derive(Debug)]
pub struct OkxMarket(pub String);

impl<StreamKind> Identifier<OkxMarket> for Subscription<OkxSpotPublicData, StreamKind> {
    fn id(&self) -> OkxMarket {
        OkxMarket(format!("{}-{}", self.instrument.base, self.instrument.quote).to_uppercase())
    }
}

impl AsRef<str> for OkxMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
