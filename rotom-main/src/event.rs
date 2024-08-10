use crate::strategy::Signal;
use rotom_data::event_models::market_event::{DataKind, MarketEvent};

pub enum Event {
    Market(MarketEvent<DataKind>),
    Signal(Signal),
}
