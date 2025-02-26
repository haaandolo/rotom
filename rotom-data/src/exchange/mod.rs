pub mod ascendex;
pub mod binance;
pub mod bitstamp;
pub mod coinex;
pub mod exmo;
pub mod htx;
pub mod kucoin;
pub mod okx;
pub mod phemex;
pub mod poloniex;
pub mod woox;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::{fmt::Debug, time::Duration};

use crate::{
    error::SocketError, model::ticker_info::TickerInfo, shared::subscription_models::{Instrument, StreamKind},
};

use super::{
    model::SubKind,
    protocols::ws::{PingInterval, WsMessage},
    shared::subscription_models::{ExchangeId, ExchangeSubscription},
    streams::validator::Validator,
    transformer::Transformer,
};

const VOLUME_THRESHOLD: u64 = 100000;
const DEFAULT_WS_STREAM_CHUNK: usize = 50;
pub const DEFAULT_SUBSCRIPTION_TIMEOUT: Duration = Duration::from_secs(10);

/*----- */
// Exchange connector trait
/*----- */
pub trait PublicStreamConnector {
    const ID: ExchangeId;
    const ORDERBOOK: StreamKind;
    const TRADE: StreamKind;

    type Channel: Send + Sync;
    type Market: Send + Sync;
    type SubscriptionResponse: DeserializeOwned + Validator + Send + Debug;

    fn url() -> impl Into<String>;

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

    fn ws_chunk_size() -> usize {
        DEFAULT_WS_STREAM_CHUNK
    }
}

/*----- */
// Exchange http connector
/*----- */
#[async_trait]
pub trait PublicHttpConnector {
    const ID: ExchangeId;
    type BookSnapShot: Send + Debug;
    type ExchangeTickerInfo: Into<TickerInfo> + Send + Debug;
    type NetworkInfo: Send + Debug;

    async fn get_book_snapshot(instrument: Instrument) -> Result<Self::BookSnapShot, SocketError>;

    async fn get_ticker_info(
        instrument: Instrument,
    ) -> Result<Self::ExchangeTickerInfo, SocketError>;

    async fn get_network_info(
        instruments: Vec<Instrument>,
    ) -> Result<Self::NetworkInfo, SocketError>;

    async fn get_usdt_pair() -> Result<Vec<(String, String)>, SocketError>;

    fn get_volume_threshold() -> u64 {
        VOLUME_THRESHOLD
    }
}

/*----- */
// Stream Selector
/*----- */
pub trait StreamSelector<Exchange, StreamKind>
where
    Exchange: PublicStreamConnector,
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
// DELETE - only here to satisfy trait req
/*----- */
impl From<serde_json::Value> for TickerInfo {
    fn from(_value: serde_json::Value) -> Self {
        unimplemented!()
    }
}
