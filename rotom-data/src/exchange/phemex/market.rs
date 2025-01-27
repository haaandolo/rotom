use crate::{exchange::Identifier, shared::subscription_models::Subscription};

use super::PhemexSpotPublicData;

#[derive(Debug)]
pub struct PhemexMarket(pub String);

impl<StreamKind> Identifier<PhemexMarket> for Subscription<PhemexSpotPublicData, StreamKind> {
    fn id(&self) -> PhemexMarket {
        PhemexMarket(format!(
            "s{}{}",
            self.instrument.base.to_uppercase(),
            self.instrument.quote.to_uppercase()
        ))
    }
}

impl AsRef<str> for PhemexMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
