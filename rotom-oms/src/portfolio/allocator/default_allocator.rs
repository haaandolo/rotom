use serde::Deserialize;

use crate::{
    model::order::{Order, RequestOpen},
    portfolio::position::Position,
};

use super::OrderAllocator;
use rotom_strategy::{Decision, SignalStrength};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Deserialize)]
pub struct DefaultAllocator {
    pub default_order_value: f64,
}

impl OrderAllocator for DefaultAllocator {
    fn allocate_order(
        &self,
        order: &mut Order<RequestOpen>,
        position: Option<&Position>,
        signal_strength: SignalStrength,
    ) {
        // Calculate exact order_size
        let default_order_size = self.default_order_value / order.state.price;

        // Then round it to a more appropriate decimal place
        let default_order_size = (default_order_size * 10000.0).floor() / 10000.0;

        match order.state.decision {
            // Entry
            Decision::Long => order.state.quantity = default_order_size * signal_strength.0,

            // Entry
            Decision::Short => order.state.quantity = -default_order_size * signal_strength.0,

            // Exit
            _ => order.state.quantity = 0.0 - position.as_ref().unwrap().quantity,
        }
    }
}
