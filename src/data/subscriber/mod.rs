use std::collections::HashMap;
use std::fmt::Debug;

use single::StreamBuilder;
use tokio::sync::mpsc::UnboundedReceiver;

use super::ExchangeId;

pub mod multi;
pub mod single;

pub struct Streams<T> {
    pub streams: HashMap<ExchangeId, UnboundedReceiver<T>>,
}

impl<T> Streams<T> {
    pub fn builder<StreamKind>() -> StreamBuilder<StreamKind>
    where
        StreamKind: Debug + Send + 'static,
    {
        StreamBuilder::<StreamKind>::new()
    }
}
