use async_trait::async_trait;
use std::fmt::Debug;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::data::models::event::MarketEvent;
use crate::data::models::SubKind;
use crate::error::SocketError;

use super::Transformer;

#[derive(Default, Clone, Eq, PartialEq, Debug, Serialize)]
pub struct StatelessTransformer<StreamKind> {
    phantom: PhantomData<StreamKind>,
}

impl<StreamKind> StatelessTransformer<StreamKind> {
    pub fn new() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<StreamKind> Transformer for StatelessTransformer<StreamKind>
where
    StreamKind: SubKind + for<'de> Deserialize<'de>,
{
    type Error = SocketError;
    type Input = StreamKind::Event;
    type Output = StreamKind::Event;

    fn transform(&mut self, input: Self::Input) -> Self::Output {
        input
    }
}

// impl<Instrument, Server> StreamSelector<Instrument, PublicTrades> for Binance<Server>
// where
//     Instrument: InstrumentData,
//     Server: ExchangeServer + Debug + Send + Sync,
// {
//     type Stream =
//         ExchangeWsStream<StatelessTransformer<Self, Instrument::Id, PublicTrades, BinanceTrade>>;
// }

// pub type ExchangeWsStream<Transformer> = ExchangeStream<WebSocketParser, WsStream, Transformer>;
