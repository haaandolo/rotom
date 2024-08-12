use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::marker::PhantomData;

use super::book::Map;
use super::{ExchangeTransformer, Transformer};
use crate::error::SocketError;
use crate::event_models::market_event::MarketEvent;
use crate::event_models::SubKind;
use crate::exchange::{Connector, Identifier};
use crate::shared::subscription_models::{ExchangeSubscription, Instrument};

/*----- */
// Stateless transformer
/*----- */
#[derive(Default, Clone, Debug)]
pub struct StatelessTransformer<Exchange, Input, Output> {
    pub instrument_map: Map<Instrument>,
    phantom: PhantomData<(Exchange, Input, Output)>,
}

/*----- */
// Impl transformer for StatelessTransformer
/*----- */
impl<Exchange, DeStruct, StreamKind> Transformer
    for StatelessTransformer<Exchange, DeStruct, StreamKind>
where
    StreamKind: SubKind,
    DeStruct: Send + for<'de> Deserialize<'de> + Identifier<String>,
    MarketEvent<StreamKind::Event>: From<(DeStruct, Instrument)>,
{
    type Error = SocketError;
    type Input = DeStruct;
    type Output = MarketEvent<StreamKind::Event>;

    fn transform(&mut self, update: Self::Input) -> Result<Self::Output, Self::Error> {
        let instrument =
            self.instrument_map
                .find_mut(&update.id())
                .ok_or(SocketError::OrderBookFindError {
                    symbol: update.id(),
                })?;
        Ok(MarketEvent::from((update, instrument.clone())))
    }
}

/*----- */
// Impl ExchangeTransformer for StatelessTransformer
/*----- */
#[async_trait]
impl<Exchange, DeStruct, StreamKind> ExchangeTransformer<Exchange, DeStruct, StreamKind>
    for StatelessTransformer<Exchange, DeStruct, StreamKind>
where
    StreamKind: SubKind,
    Exchange: Connector + Sync,
    Exchange::Market: AsRef<str>,
    DeStruct: Send + for<'de> Deserialize<'de> + Identifier<String>,
    MarketEvent<StreamKind::Event>: From<(DeStruct, Instrument)>,
{
    async fn new(
        subs: &[ExchangeSubscription<Exchange, Exchange::Channel, Exchange::Market>],
    ) -> Result<Self, SocketError> {
        let instrument_map = subs
            .iter()
            .map(|sub| (String::from(sub.market.as_ref()), sub.instrument.clone()))
            .collect::<HashMap<String, Instrument>>();
        Ok(Self {
            instrument_map: Map(instrument_map),
            phantom: PhantomData,
        })
    }
}
