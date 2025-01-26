use crate::{exchange::Identifier, shared::subscription_models::Subscription};

use super::AscendExSpotPublicData;


#[derive(Debug)]
pub struct AscendExMarket(pub String);

impl<StreamKind> Identifier<AscendExMarket> for Subscription<AscendExSpotPublicData, StreamKind> {
    fn id(&self) -> AscendExMarket {
        AscendExMarket(format!("{}/{}", self.instrument.base, self.instrument.quote).to_uppercase())
    }
}

impl AsRef<str> for AscendExMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
