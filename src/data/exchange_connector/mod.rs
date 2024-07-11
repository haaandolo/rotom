pub mod binance;
pub mod poloniex;

use std::fmt::Debug;

use serde::de::DeserializeOwned;

use super::{
    protocols::ws::ws_client::{PingInterval, WsMessage},
    shared::orderbook::Event,
    ExchangeId, Instrument,
};

/*----- */
// Exchange connector trait
/*----- */
pub trait Connector {
    type ExchangeId;
    type SubscriptionResponse;

    const ID: ExchangeId;

    fn url() -> String;

    fn ping_interval() -> Option<PingInterval> {
        None
    }

    fn requests(subscriptions: &[Instrument]) -> Option<WsMessage>;
    
    fn validate_subscription(
        subscription_repsonse: String,
        subscriptions: &[Instrument],
    ) -> bool;
}

/*----- */
// Stream Selector
/*----- */
pub trait StreamSelector<Exchange, Kind>
where
    Exchange: Connector,
{
    type Stream: DeserializeOwned + Into<Event> + Debug;
}

/*----- */
// Identifier
/*----- */
pub trait Identifier<T> {
    fn id(&self) -> T;
}