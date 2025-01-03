pub mod binance;
pub mod poloniex;

use std::{fmt::Debug, time::Duration};

use serde::de::DeserializeOwned;

use super::{
    event_models::SubKind,
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    streams::validator::Validator,
    transformer::Transformer,
};

pub const DEFAULT_SUBSCRIPTION_TIMEOUT: Duration = Duration::from_secs(10);

/*----- */
// Exchange connector trait
/*----- */
pub trait Connector {
    type ExchangeId;
    type Channel: Send + Sync;
    type Market: Send + Sync;
    type SubscriptionResponse: DeserializeOwned + Validator + Send + Debug;

    const ID: ExchangeId;

    fn url() -> &'static str;

    fn ping_interval() -> Option<PingInterval> {
        None
    }

    fn requests(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> Option<WsMessage>
    where
        Self: Sized;

    fn expected_responses(
        subscriptions: &[ExchangeSubscription<Self, Self::Channel, Self::Market>],
    ) -> usize
    where
        Self: Sized,
    {
        subscriptions.len()
    }

    fn subscription_validation_timeout() -> Duration {
        DEFAULT_SUBSCRIPTION_TIMEOUT
    }
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

/*----- */
// Trait to get relevant fields from ticker info - used for execution client
/*----- */
pub trait TickerInfo {
    // Represents how much the quantity of base asset is allowed to +ve or -ve
    // E.g. 0.01, this means the base asset quanitity has to be fixed at 2 dp
    // This is useful for opening new orders. Some exchanges give precision value
    // as int's i.e. 2 means 0.01 so we need to make the required coversions
    fn get_asset_quantity_precision(&self) -> f64;

    // Represents how much the price of base asset is allowed to +ve or -ve
    // E.g. 0.01, this means the base asset price can tick by intervals of
    // 0.01 (1.41 <- 1.42 <- # 1.43 # -> 1.44 -> 1.45). This is useful when
    // sending market orders and you want to send orders in usdt i.e. the
    // quote asset price and not the base asset quantity. Some exchanges give
    // precision value as int's i.e. 2 means 0.01 so we need to make the
    // required coversions
    fn get_asset_price_precision(&self) -> f64;
}
