use crate::{exchange::Identifier, shared::subscription_models::Subscription};

use super::WooxSpotPublicData;

#[derive(Debug)]
pub struct WooxMarket(pub String);

impl<StreamKind> Identifier<WooxMarket> for Subscription<WooxSpotPublicData, StreamKind> {
    fn id(&self) -> WooxMarket {
        WooxMarket(
            format!("SPOT_{}_{}", self.instrument.base, self.instrument.quote).to_uppercase(),
        )
    }
}

impl AsRef<str> for WooxMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
