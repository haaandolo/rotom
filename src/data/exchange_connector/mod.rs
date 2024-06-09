use super::{protocols::ws::{PingInterval, WsMessage}, ExchangeSub};

pub mod binance;
pub mod poloniex;

/*-------- */
// Exchange connector trait
/*-------- */
pub trait Connector {
    fn url(&self) -> String;

    fn ping_interval(&self) -> Option<PingInterval> {
        None
    }

    fn requests(&self, subscriptions: &[ExchangeSub]) -> Option<WsMessage>;

    fn expected_response(&self) -> Option<usize> {
        None
    }
}