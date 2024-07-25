use std::marker::PhantomData;
use serde::{Deserialize, Serialize};

use crate::data::model::event::MarketEvent;
use crate::data::model::SubKind;
use crate::error::SocketError;

use super::Transformer;

#[derive(Default, Clone, Eq, PartialEq, Debug, Serialize)]
pub struct StatelessTransformer<Input, Output> {
    phantom: PhantomData<(Input, Output)>,
}

impl<Input, Output> StatelessTransformer<Input, Output> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

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
