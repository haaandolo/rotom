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

impl WsStatus {
    pub fn is_connected(&self) -> bool {
        match self {
            WsStatus::Connected(_) => true,
            WsStatus::Disconnected(_) => false,
        }
    }

    pub fn get_event_kind(&self) -> EventKind {
        match self {
            WsStatus::Connected(event_kind) => *event_kind,
            WsStatus::Disconnected(event_kind) => *event_kind,
        }
    }
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

impl DataKind {
    // Used for testing in #[cfg(test)]
    pub fn get_trade(&self) -> Option<EventTrade> {
        if let DataKind::Trade(trade) = self {
            Some(trade.to_owned())
        } else {
            None
        }
    }

    // Used for testing in #[cfg(test)]
    pub fn get_trades(&self) -> Option<Vec<EventTrade>> {
        if let DataKind::Trades(trades) = self {
            Some(trades.to_owned())
        } else {
            None
        }
    }

    // Used for testing in #[cfg(test)]
    pub fn get_orderbook(&self) -> Option<EventOrderBook> {
        if let DataKind::OrderBook(orderbook) = self {
            Some(orderbook.to_owned())
        } else {
            None
        }
    }
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
