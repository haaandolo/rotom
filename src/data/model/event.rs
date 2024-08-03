use super::{
    event_book::EventOrderBook,
    event_trade::EventTrade,
    subs::{ExchangeId, StreamType},
};

#[derive(Debug)]
pub enum DataKind {
    Trade(EventTrade),
    OrderBook(EventOrderBook),
    Default,
}

#[derive(Debug)]
pub struct MarketEvent<Event> {
    pub exchange_time: u64,
    pub received_time: u64,
    pub seq: u64,
    pub exchange: ExchangeId,
    pub stream_type: StreamType,
    pub symbol: String,
    pub event_data: Event,
}

impl<Event> Default for MarketEvent<Event>
where
    Event: Default,
{
    fn default() -> Self {
        Self {
            exchange_time: 0,
            received_time: 0,
            seq: 0,
            exchange: ExchangeId::Default,
            stream_type: StreamType::Default,
            symbol: String::new(),
            event_data: Event::default(),
        }
    }
}

impl From<MarketEvent<EventTrade>> for MarketEvent<DataKind> {
    fn from(event: MarketEvent<EventTrade>) -> Self {
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
