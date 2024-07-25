use super::{
    book::EventOrderBook,
    subs::{ExchangeId, StreamType},
    trade::Trade,
};

#[derive(Debug)]
pub enum DataKind {
    Trade(Trade),
    OrderBook(EventOrderBook),
}

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

impl From<MarketEvent<Trade>> for MarketEvent<DataKind> {
    fn from(event: MarketEvent<Trade>) -> Self {
        Self {
            exchange_time: event.exchange_time,
            received_time: event.received_time,
            exchange: event.exchange,
            seq: event.seq,
            stream_type: event.stream_type,
            symbol: event.symbol,
            event_data: DataKind::Trade(event.event_data),
        }
    }
}

impl From<MarketEvent<EventOrderBook>> for MarketEvent<DataKind> {
    fn from(event: MarketEvent<EventOrderBook>) -> Self {
        Self {
            exchange_time: event.exchange_time,
            received_time: event.received_time,
            exchange: event.exchange,
            seq: event.seq,
            stream_type: event.stream_type,
            symbol: event.symbol,
            event_data: DataKind::OrderBook(event.event_data),
        }
    }
}
