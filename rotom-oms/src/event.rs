use crate::{
    execution::FillEvent,
    model::{
        balance::Balance,
        account::{AccountBalance, AccountBalanceDelta, OrderResponse},
        order::OrderEvent,
    },
    portfolio::position::{Position, PositionExit, PositionUpdate},
};
use rotom_data::model::market_event::{DataKind, MarketEvent};
use rotom_strategy::{Signal, SignalForceExit};
use tokio::sync::mpsc;
use tracing::warn;

#[derive(Debug)]
pub enum Event {
    Market(MarketEvent<DataKind>),
    Signal(Signal),
    SignalForceExit(SignalForceExit),
    OrderEvaluate(OrderEvent),
    OrderNew(OrderEvent),
    OrderUpdate,
    Fill(FillEvent),
    PositionNew(Position),
    PositionUpdate(PositionUpdate),
    PositionExit(PositionExit),
    Balance(Balance),
    OrderResponse(OrderResponse),
    AccountDataBalanceVec(Vec<AccountBalance>),
    AccountBalance(AccountBalance),
    AccountBalanceDelta(AccountBalanceDelta),
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
