use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::marker::PhantomData;

use super::book::Map;
use super::{ExchangeTransformer, Transformer};
use crate::error::SocketError;
use crate::event_models::market_event::MarketEvent;
use crate::event_models::SubKind;
use crate::exchange::Connector;
use crate::shared::subscription_models::{ExchangeSubscription, Instrument};

/*----- */
// Stateless transformer
/*----- */
#[derive(Default, Clone, Debug)]
pub struct StatelessTransformer<Input, Output> {
    pub instrument_map: Map<Instrument>,
    phantom: PhantomData<(Input, Output)>,
}

/*----- */
// Impl transformer for StatelessTransformer
/*----- */
impl<DeStruct, StreamKind> Transformer for StatelessTransformer<DeStruct, StreamKind>
where
    DeStruct: Send + for<'de> Deserialize<'de> + Into<MarketEvent<StreamKind::Event>>,
    StreamKind: SubKind,
{
    type Error = SocketError;
    type Input = DeStruct;
    type Output = MarketEvent<StreamKind::Event>;

    fn transform(&mut self, update: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(update.into())
    }
}

/*----- */
// Impl ExchangeTransformer for StatelessTransformer
/*----- */
#[async_trait]
impl<Exchange, DeStruct, StreamKind> ExchangeTransformer<Exchange, DeStruct, StreamKind>
    for StatelessTransformer<DeStruct, StreamKind>
where
    Exchange: Connector + Sync,
    StreamKind: SubKind,
    DeStruct: Send + for<'de> Deserialize<'de>,
    MarketEvent<StreamKind::Event>: From<DeStruct>,
{
    async fn new(
        subs: &[ExchangeSubscription<Exchange, Exchange::Channel, Exchange::Market>],
    ) -> Result<Self, SocketError> {
        let instrument_map = Map(HashMap::with_capacity(subs.len()));
        Ok(Self {
            instrument_map,
            phantom: PhantomData,
        })
    }
}
