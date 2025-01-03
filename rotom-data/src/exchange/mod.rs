pub mod binance;
pub mod poloniex;

use std::{fmt::Debug, time::Duration};

use serde::de::DeserializeOwned;

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
pub trait Connector {
    type ExchangeId;
    type Channel: Send + Sync;
    type Market: Send + Sync;
    type SubscriptionResponse: DeserializeOwned + Validator + Send + Debug;

    const ID: ExchangeId;

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
// Stream Selector
/*----- */
pub trait StreamSelector<Exchange, StreamKind>
where
    Exchange: Connector,
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
