use crate::{
    execution::FillEvent,
    oms::{
        position::{Position, PositionExit, PositionUpdate},
        Balance, OrderEvent,
    },
    strategy::{Signal, SignalForceExit},
};
use rotom_data::event_models::market_event::{DataKind, MarketEvent};

pub enum Event {
    Market(MarketEvent<DataKind>),
    Signal(Signal),
    SignalForceExit(SignalForceExit),
    OrderNew(OrderEvent),
    OrderUpdate,
    Fill(FillEvent),
    PositionNew(Position),
    PositionUpdate(PositionUpdate),
    PositionExit(PositionExit),
    Balance(Balance),
}
