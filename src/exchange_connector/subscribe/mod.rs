use std::collections::HashMap;

use super::{protocols::ws::WsRead, Subscription};
use crate::exchange_connector::{Exchange, FuturesTokio};

pub struct ExchangeStream {
    pub stream: WsRead,
    pub tasks: Vec<FuturesTokio>,
}

pub struct StreamBuilder {
    pub streams: HashMap<Exchange, ExchangeStream>,
}

impl Default for StreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamBuilder {

    pub fn new() -> Self {
        Self {
            streams: HashMap::new()
        }
    }

    // pub fn subscribe(mut self, subscription: impl Iterator) -> Self {

    //     self
    // }
}
