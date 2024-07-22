use async_trait::async_trait;
use std::fmt::Debug;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::data::models::SubKind;
use crate::error::SocketError;

use super::Transformer;

#[derive(Default, Clone, Eq, PartialEq, Debug, Serialize)]
pub struct StatelessTransformer<DeStruct> {
    phantom: PhantomData<DeStruct>,
}

impl<DeStruct> StatelessTransformer<DeStruct> {
    pub fn new() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<DeStruct> Transformer for StatelessTransformer<DeStruct>
where
    DeStruct: Send + Debug + for<'de> Deserialize<'de>,
{
    type Error = SocketError;
    type Input = DeStruct;
    type Output = DeStruct;

    fn transform(&mut self, input: Self::Input) -> Self::Output {
        input
    }
}