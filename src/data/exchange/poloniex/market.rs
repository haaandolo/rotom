use super::PoloniexSpot;
use crate::data::{exchange::Identifier, model::subs::Subscription};

#[derive(Debug)]
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
