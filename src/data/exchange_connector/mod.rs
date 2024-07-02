pub mod binance;
pub mod poloniex;

use serde::Deserialize;
use std::{fmt::Debug, time::Duration};

use super::{
    protocols::ws::ws_client::{PingInterval, WsMessage}, Instrument
};

/*----- */
// Exchange connector trait
/*----- */
pub trait Connector {
    type ExchangeId;
    type Input: for<'de> Deserialize<'de> + Debug;
    type Output: Send;
    type SubscriptionResponse;

    fn exchange_id(&self) -> String;

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

    fn subscription_timeout() -> Duration {
        Duration::from_secs(10)
    }
}
