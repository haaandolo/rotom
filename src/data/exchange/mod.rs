pub mod binance;
pub mod poloniex;

use std::fmt::Debug;

use serde::de::DeserializeOwned;

use super::{
    models::{
        event::MarketEvent,
        subs::{ExchangeId, Instrument},
        SubKind,
    },
    protocols::ws::ws_client::{PingInterval, WsMessage}, transformer::Transformer,
};

/*----- */
// Exchange connector trait
/*----- */
pub trait Connector {
    type ExchangeId;
    type SubscriptionResponse: DeserializeOwned;
    // type Channel: AsRef<str>;
    // type Market: AsRef<str>;

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
    type Stream: DeserializeOwned + Into<MarketEvent<StreamKind::Event>> + Debug + Send;
    type StreamTransformer: Transformer + Default + Send;
}

/*----- */
// Identifier
/*----- */
pub trait Identifier<T> {
    fn id(&self) -> T;
}
