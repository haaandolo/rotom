pub mod book;
pub mod stateless_transformer;

use async_trait::async_trait;
use serde::Deserialize;

use crate::{exchange::PublicStreamConnector, shared::subscription_models::ExchangeSubscription};

use super::{
    error::SocketError,
    model::{market_event::MarketEvent, SubKind},
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
pub trait ExchangeTransformer<Exchange, DeStruct, StreamKind>
where
    Self: Transformer<Input = DeStruct, Output = MarketEvent<StreamKind::Event>, Error = SocketError>
        + Sized,
    StreamKind: SubKind,
    Exchange: PublicStreamConnector,
{
    async fn new(
        subs: &[ExchangeSubscription<Exchange, Exchange::Channel, Exchange::Market>],
    ) -> Result<Self, SocketError>;
}
