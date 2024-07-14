use super::subs::{ExchangeId, StreamType};

#[derive(Debug)]
pub struct MarketEvent<T> {
    pub exchange_time: u64,
    pub received_time: u64,
    pub seq: u64,
    pub exchange: ExchangeId,
    pub stream_type: StreamType,
    pub symbol: String,
    pub event_data: T,
}