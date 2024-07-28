use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::marker::PhantomData;

use super::book::Map;
use super::{ExchangeTransformer, Transformer};
use crate::data::model::event::MarketEvent;
use crate::data::model::subs::Instrument;
use crate::data::model::SubKind;
use crate::error::SocketError;

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
impl<DeStruct, StreamKind> ExchangeTransformer<DeStruct, StreamKind>
    for StatelessTransformer<DeStruct, StreamKind>
where
    StreamKind: SubKind,
    DeStruct: Send + for<'de> Deserialize<'de>,
    MarketEvent<StreamKind::Event>: From<DeStruct>,
{
    async fn new(subs: &[Instrument]) -> Result<Self, SocketError> {
        let instrument_map = Map(HashMap::with_capacity(subs.len()));
        Ok(Self {
            instrument_map,
            phantom: PhantomData,
        })
    }
}
