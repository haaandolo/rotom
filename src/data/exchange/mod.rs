pub mod binance;
pub mod poloniex;

use serde::de::DeserializeOwned;

use super::{
    model::{
        subs::{ExchangeId, ExchangeSubscription},
        SubKind,
    },
    protocols::ws::{PingInterval, WsMessage},
    transformer::Transformer,
};

/*----- */
// Exchange connector trait
/*----- */
pub trait Connector {
    type ExchangeId;
    type SubscriptionResponse: DeserializeOwned;
    type Channel: Send;
    type Market: Send;

    const ID: ExchangeId;

    fn url() -> String;

    fn ping_interval() -> Option<PingInterval> {
        None
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage>
    where
        Self: Sized;

    fn validate_subscription(subscription_repsonse: String, number_of_tickers: usize) -> bool
    where
        Self: Sized;
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
