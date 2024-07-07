pub mod binance;
pub mod poloniex;

use serde::Deserialize;
use std::fmt::Debug;

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
    type Input: for<'de> Deserialize<'de> + Debug;
    type Output: Send;
    type SubscriptionResponse;

    fn exchange_id(&self) -> ExchangeId;

    fn url(&self) -> String;

    fn ping_interval(&self) -> Option<PingInterval> {
        None
    }

    fn requests(&self, subscriptions: &[Instrument]) -> Option<WsMessage>;

    fn validate_subscription(
        &self,
        subscription_repsonse: String,
        subscriptions: &[Instrument],
    ) -> bool;

    fn transform(&mut self, input: Self::Input) -> Self::Output;
}

/*----- */
// Subscription kinds
/*----- */
pub trait SubKind
where
    Self: Debug + Clone,
{
    type Event: Debug;
}

#[derive(Clone, Debug)]
pub struct Trades;

impl SubKind for Trades {
    type Event = Event;
}

#[derive(Clone, Debug)]
pub struct L2;

impl SubKind for L2 {
    type Event = Event;
}
/*----- */
// Stream Selector
/*----- */
pub trait ExchangeSelector
{
    type ExchangeConn: Connector;
}
