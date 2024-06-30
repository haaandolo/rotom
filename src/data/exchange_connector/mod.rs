pub mod binance;
pub mod poloniex;

use std::fmt::Debug;

use serde::Deserialize;

use super::{
    protocols::ws::{PingInterval, WsMessage}, shared::orderbook::Event, ExchangeSub
};

/*----- */
// Exchange connector trait
/*----- */
pub trait Connector {
    type ExchangeId;
    type Input: for<'de> Deserialize<'de> + Debug;
    type Output: Send;

    fn exchange_id(&self) -> String;

    fn url(&self) -> String;

    fn ping_interval(&self) -> Option<PingInterval> {
        None
    }

    fn requests(&self, subscriptions: &[ExchangeSub]) -> Option<WsMessage>;

    fn expected_response(
        &self,
        subscription_repsonse: String,
        subscriptions: &[ExchangeSub],
    ) -> bool;

    fn transform(&mut self, input: Self::Input) -> Self::Output;
}
