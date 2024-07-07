pub mod binance;
pub mod poloniex;


use super::{
    protocols::ws::ws_client::{PingInterval, WsMessage}, ExchangeId, Instrument
};

/*----- */
// Exchange connector trait
/*----- */
pub trait Connector {
    type ExchangeId;
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
}
