pub mod binance;
pub mod poloniex;

use super::{
    protocols::ws::{PingInterval, WsMessage},
    ExchangeSub,
};

/*----- */
// Exchange connector trait
/*----- */
pub trait Connector {
    type ExchangeId;
    type Message;

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
}
