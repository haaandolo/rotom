use serde::Deserialize;

use crate::{
    model::order::{Order, RequestOpen},
    portfolio::position::Position,
};

use rotom_strategy::{Decision, SignalStrength};

use super::OrderAllocator;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Deserialize)]
pub struct SpotArbAllocator;

impl OrderAllocator for SpotArbAllocator {
    fn allocate_order(
        &self,
        order: &mut Order<RequestOpen>,
        position: Option<&Position>,
        signal_strength: SignalStrength,
    ) {
        // Calculate exact order_size
        let dollar_amount = order.state.price * signal_strength.0;
        let order_size = dollar_amount / order.state.price;

        match order.state.decision {
            // Entry
            Decision::Long => order.state.quantity = order_size,

            // Entry
            Decision::Short => order.state.quantity = -order_size,

            // Exit
            _ => order.state.quantity = 0.0 - position.as_ref().unwrap().quantity,
        }
    }
}
