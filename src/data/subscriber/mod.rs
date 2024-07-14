use std::collections::HashMap;
use std::fmt::Debug;

use single::StreamBuilder;
use tokio::sync::mpsc::UnboundedReceiver;

use super::models::subs::ExchangeId;
use super::models::SubKind;

pub mod multi;
pub mod single;

pub struct Streams<T> {
    pub streams: HashMap<ExchangeId, UnboundedReceiver<T>>,
}

impl<T> Streams<T> {
    pub fn builder<StreamKind>() -> StreamBuilder<StreamKind>
    where
        StreamKind: SubKind + Debug + Send + 'static,
    {
        StreamBuilder::<StreamKind>::new()
    }
}
