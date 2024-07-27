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
impl<Input, Output> Transformer for StatelessTransformer<Input, Output>
where
    Input: Send + for<'de> Deserialize<'de> + Into<MarketEvent<Output::Event>>,
    Output: SubKind,
{
    type Error = SocketError;
    type Input = Input;
    type Output = MarketEvent<Output::Event>;

    fn transform(&mut self, input: Self::Input) -> Self::Output {
        input.into()
    }
}

/*----- */
// Impl ExchangeTransformer for StatelessTransformer
/*----- */
#[async_trait]
impl<Input, Output> ExchangeTransformer<Input, Output> for StatelessTransformer<Input, Output>
where
    Output: SubKind,
    Input: Send + for<'de> Deserialize<'de>,
    MarketEvent<Output::Event>: From<Input>,
{
    async fn new(subs: &[Instrument]) -> Result<Self, SocketError> {
        let instrument_map = Map(HashMap::with_capacity(subs.len()));
        Ok(Self {
            instrument_map,
            phantom: PhantomData,
        })
    }
}
