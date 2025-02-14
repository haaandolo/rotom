use crate::{exchange::Identifier, shared::subscription_models::Subscription};

use super::HtxSpotPublicData;

#[derive(Debug)]
pub struct HtxMarket(pub String);

impl<StreamKind> Identifier<HtxMarket> for Subscription<HtxSpotPublicData, StreamKind> {
    fn id(&self) -> HtxMarket {
        HtxMarket(format!("{}{}", self.instrument.base, self.instrument.quote).to_lowercase())
    }
}

impl AsRef<str> for HtxMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
