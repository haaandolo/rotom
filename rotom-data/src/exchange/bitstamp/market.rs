use crate::{exchange::Identifier, shared::subscription_models::Subscription};

#[derive(Debug)]
pub struct BitstampMarket(pub String);

impl<StreamKind> Identifier<BitstampMarket> for Subscription<BitstampMarket, StreamKind> {
    fn id(&self) -> BitstampMarket {
        BitstampMarket(format!("{}{}", self.instrument.base, self.instrument.quote))
    }
}

impl AsRef<str> for BitstampMarket {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
