pub mod book;
pub mod stateless_transformer;

use async_trait::async_trait;
use serde::Deserialize;

use super::{
    error::SocketError,
    event_models::{event::MarketEvent,  SubKind}, shared::subscription_models::Instrument
};

/*----- */
// WebSocket transformer
/*----- */
pub trait Transformer {
    type Error: Send;
    type Input: for<'de> Deserialize<'de>;
    type Output: Send;

    fn transform(&mut self, update: Self::Input) -> Result<Self::Output, Self::Error>;
}

/*----- */
// Exchange transformer
/*----- */
#[async_trait]
pub trait ExchangeTransformer<DeStruct, StreamKind>
where
    Self: Transformer<Input = DeStruct, Output = MarketEvent<StreamKind::Event>, Error = SocketError>
        + Sized,
    StreamKind: SubKind,
{
    async fn new(subs: &[Instrument]) -> Result<Self, SocketError>;
}
