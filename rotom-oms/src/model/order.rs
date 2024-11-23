use rotom_data::shared::subscription_models::ExchangeId;
use serde::{Deserialize, Serialize};

use super::{ClientOrderId, OrderId, OrderKind, Side};

/*----- */
// Order
/*----- */
#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Order<State> {
    pub exchange: ExchangeId,
    pub instrument: String, // todo change?
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

/*----- */
// Order - RequestCancel
/*----- */
pub struct RequestCancel {
    pub id: OrderId,
}

/*----- */
// Open - order has been open
/*----- */
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Open {
    pub id: u64,
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
