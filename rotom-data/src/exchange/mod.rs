pub mod binance;
pub mod poloniex;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::{fmt::Debug, time::Duration};

use crate::{
    error::SocketError, model::ticker_info::TickerInfo, shared::subscription_models::Instrument,
};

use super::{
    model::SubKind,
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    streams::validator::Validator,
    transformer::Transformer,
};

pub const DEFAULT_SUBSCRIPTION_TIMEOUT: Duration = Duration::from_secs(10);

/*----- */
// Exchange connector trait
/*----- */
pub trait PublicStreamConnector {
    const ID: ExchangeId;

    type Channel: Send + Sync;
    type Market: Send + Sync;
    type SubscriptionResponse: DeserializeOwned + Validator + Send + Debug;

    fn url() -> &'static str;

    fn ping_interval() -> Option<PingInterval> {
        None
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage>
    where
        Self: Sized;

    fn expected_responses(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> usize
    where
        Self: Sized,
    {
        subscriptions.len()
    }

    fn subscription_validation_timeout() -> Duration {
        DEFAULT_SUBSCRIPTION_TIMEOUT
    }
}

/*----- */
// Exchange http connector
/*----- */
#[async_trait]
pub trait PublicHttpConnector {
    type BookSnapShot: Send + Debug;
    type ExchangeTickerInfo: Into<TickerInfo> + Send + Debug;

    const ID: ExchangeId;

    async fn get_book_snapshot(instrument: Instrument) -> Result<Self::BookSnapShot, SocketError>;

    async fn get_ticker_info(
        instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError>;
}

/*----- */
// Stream Selector
/*----- */
pub trait StreamSelector<Exchange, StreamKind>
where
    Exchange: PublicStreamConnector,
    StreamKind: SubKind,
{
    type Stream;
    type StreamTransformer: Transformer + Default + Send;
}

/*----- */
// Identifier
/*----- */
pub trait Identifier<T> {
    fn id(&self) -> T;
}
