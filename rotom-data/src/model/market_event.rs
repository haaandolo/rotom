use chrono::{DateTime, Utc};

use crate::shared::subscription_models::{ExchangeId, Instrument};

use super::{
    event_book::EventOrderBook, event_book_snapshot::EventOrderBookSnapshot,
    event_trade::EventTrade,
};

#[derive(Debug)]
pub enum DataKind {
    Trade(EventTrade),
    OrderBook(EventOrderBook),
    OrderBookSnapshot(EventOrderBookSnapshot),
}

#[derive(Debug)]
pub struct MarketEvent<Event> {
    pub exchange_time: DateTime<Utc>,
    pub received_time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub instrument: Instrument,
    pub event_data: Event,
}

impl From<MarketEvent<EventTrade>> for MarketEvent<DataKind> {
    fn from(event: MarketEvent<EventTrade>) -> Self {
        Self {
            exchange_time: event.exchange_time,
            received_time: event.received_time,
            exchange: event.exchange,
            instrument: event.instrument,
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
            instrument: event.instrument,
            event_data: DataKind::OrderBook(event.event_data),
        }
    }
}

impl From<MarketEvent<EventOrderBookSnapshot>> for MarketEvent<DataKind> {
    fn from(event: MarketEvent<EventOrderBookSnapshot>) -> Self {
        Self {
            exchange_time: event.exchange_time,
            received_time: event.received_time,
            exchange: event.exchange,
            instrument: event.instrument,
            event_data: DataKind::OrderBookSnapshot(event.event_data),
        }
    }
}
