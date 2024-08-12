use crate::{oms::OrderEvent, strategy::Signal};
use rotom_data::event_models::market_event::{DataKind, MarketEvent};

pub enum Event {
    Market(MarketEvent<DataKind>),
    Signal(Signal),
    OrderNew(OrderEvent)
}
