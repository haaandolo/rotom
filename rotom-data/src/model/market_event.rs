use chrono::{DateTime, Utc};

use crate::shared::subscription_models::{ExchangeId, Instrument};

use super::{
    event_book::EventOrderBook, event_book_snapshot::EventOrderBookSnapshot,
    event_trade::EventTrade, EventKind,
};

/*----- */
// Connection status
/*----- */
#[derive(Debug)]
pub enum WsStatus {
    Connected(EventKind),
    Disconnected(EventKind),
}

/*----- */
// DataKind
/*----- */
#[derive(Debug)]
pub enum DataKind {
    Trade(EventTrade),
    Trades(Vec<EventTrade>),
    OrderBook(EventOrderBook),
    OrderBookSnapshot(EventOrderBookSnapshot),
    ConnectionStatus(WsStatus),
}

/*----- */
// Market Event - Generic
/*----- */
#[derive(Debug)]
pub struct MarketEvent<Event> {
    pub exchange_time: DateTime<Utc>,
    pub received_time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub instrument: Instrument,
    pub event_data: Event,
}

/*----- */
// Market Event - Datakind
/*----- */
impl MarketEvent<WsStatus> {
    pub fn new_connected(
        exchange: ExchangeId,
        instrument: Instrument,
        event_kind: EventKind,
    ) -> MarketEvent<WsStatus> {
        Self {
            exchange_time: Utc::now(),
            received_time: Utc::now(),
            exchange,
            instrument,
            event_data: WsStatus::Connected(event_kind),
        }
    }

    pub fn new_disconnected(
        exchange: ExchangeId,
        instrument: Instrument,
        event_kind: EventKind,
    ) -> MarketEvent<WsStatus> {
        Self {
            exchange_time: Utc::now(),
            received_time: Utc::now(),
            exchange,
            instrument,
            event_data: WsStatus::Disconnected(event_kind),
        }
    }
}

impl From<MarketEvent<WsStatus>> for MarketEvent<DataKind> {
    fn from(event: MarketEvent<WsStatus>) -> Self {
        Self {
            exchange_time: event.exchange_time,
            received_time: event.received_time,
            exchange: event.exchange,
            instrument: event.instrument,
            event_data: DataKind::ConnectionStatus(event.event_data),
        }
    }
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

impl From<MarketEvent<Vec<EventTrade>>> for MarketEvent<DataKind> {
    fn from(event: MarketEvent<Vec<EventTrade>>) -> Self {
        Self {
            exchange_time: event.exchange_time,
            received_time: event.received_time,
            exchange: event.exchange,
            instrument: event.instrument,
            event_data: DataKind::Trades(event.event_data),
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
