use crate::{
    execution::FillEvent,
    model::balance::Balance,
    portfolio::{
        position::{Position, PositionExit, PositionUpdate},
        OrderEvent,
    },
};
use rotom_data::event_models::market_event::{DataKind, MarketEvent};
use rotom_strategy::{Signal, SignalForceExit};
use tokio::sync::mpsc;
use tracing::warn;

#[derive(Debug)]
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

pub trait MessageTransmitter<Message> {
    fn send(&mut self, message: Message);

    fn send_many(&mut self, messages: Vec<Message>);
}

#[derive(Debug, Clone)]
pub struct EventTx {
    receiver_dropped: bool,
    event_tx: mpsc::UnboundedSender<Event>,
}

impl MessageTransmitter<Event> for EventTx {
    fn send(&mut self, message: Event) {
        if self.receiver_dropped {
            return;
        }

        if self.event_tx.send(message).is_err() {
            warn!(
                action = "setting receiver_dropped = true",
                why = "event receiver dropped",
                "cannot send Events"
            );
            self.receiver_dropped = true;
        }
    }

    fn send_many(&mut self, messages: Vec<Event>) {
        if self.receiver_dropped {
            return;
        }

        messages.into_iter().for_each(|message| {
            let _ = self.event_tx.send(message);
        })
    }
}

impl EventTx {
    pub fn new(event_tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            receiver_dropped: false,
            event_tx,
        }
    }
}
