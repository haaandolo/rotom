use crate::data::{exchange::Identifier, models::subs::Subscription};

use super::PoloniexSpot;

pub struct PoloniexMarket(pub String);

impl<StreamKind> Identifier<PoloniexMarket> for Subscription<PoloniexSpot, StreamKind> {
    fn id(&self) -> PoloniexMarket {
        PoloniexMarket(format!("{}_{}", self.instrument.base, self.instrument.quote).to_lowercase())
    }
}

impl AsRef<str> for PoloniexMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
