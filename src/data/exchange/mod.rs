pub mod binance;
pub mod poloniex;

use serde::de::DeserializeOwned;

use super::{
    models::{
        subs::{ExchangeId, Instrument},
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

    const ID: ExchangeId;

    fn url() -> String;

    fn ping_interval() -> Option<PingInterval> {
        None
    }

    fn requests(subscriptions: &[Instrument]) -> Option<WsMessage>;

    fn validate_subscription(subscription_repsonse: String, subscriptions: &[Instrument]) -> bool;
}

/*----- */
// Stream Selector
/*----- */
pub trait StreamSelector<Exchange, StreamKind>
where
    Exchange: Connector,
    StreamKind: SubKind,
{
    type StreamTransformer: Transformer + Default + Send;
}

/*----- */
// Identifier
/*----- */
pub trait Identifier<T> {
    fn id(&self) -> T;
}
