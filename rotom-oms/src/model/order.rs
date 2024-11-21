use std::fmt::{Display, Formatter};

use rotom_data::shared::subscription_models::{ExchangeId, Instrument};
use serde::{Deserialize, Serialize};

use super::{ClientOrderId, Side};

/*----- */
// Order Kind
/*----- */
pub enum OrderKind {
    Market,
    Limit,
}

impl Display for OrderKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderKind::Market => "market",
                OrderKind::Limit => "limit",
            }
        )
    }
}

#[derive(Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct OrderId(pub String); // can be smolstr

/*----- */
// Order
/*----- */
#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Order<State> {
    pub exchange: ExchangeId,
    pub instrument: Instrument, // todo change?
    pub client_order_id: ClientOrderId,
    pub side: Side, // todo check duplicated enum
    pub state: State,
}

/*----- */
// Order - RequestOpen
/*----- */
pub struct RequestOpen {
    pub kind: OrderKind,
    pub price: f64,
    pub quantity: f64,
}

impl Order<RequestOpen> {
    pub fn required_available_balance(&self) -> (&String, f64) {
        match self.side {
            Side::Buy => (
                &self.instrument.quote,
                self.state.price * self.state.quantity,
            ),
            Side::Sell => (&self.instrument.base, self.state.quantity),
        }
    }
}

/*----- */
// Order - RequestCancel
/*----- */
pub struct RequestCancel {
    pub id: OrderId,
}

/*----- */
// InFlight - when an order is in  the network and has not been confirmed
/*----- */
pub struct InFlight;

/*----- */
// Open - order has been open
/*----- */
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Open {
    pub id: OrderId,
    pub price: f64,
    pub quantity: f64,
    pub filled_quantity: f64,
}

impl Open {
    pub fn remaining_quantity(&self) -> f64 {
        self.quantity - self.filled_quantity
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub enum OrderFill {
    Full,
    Partial,
}
