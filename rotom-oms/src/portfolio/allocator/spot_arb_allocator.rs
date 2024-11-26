use serde::Deserialize;

use crate::{model::order::OrderEvent, portfolio::position::Position};

use rotom_strategy::{Decision, SignalStrength};

use super::OrderAllocator;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Deserialize)]
pub struct SpotArbAllocator;

impl OrderAllocator for SpotArbAllocator {
    fn allocate_order(
        &self,
        order: &mut OrderEvent,
        position: Option<&Position>,
        signal_strength: SignalStrength,
    ) {
        // Calculate exact order_size
        let dollar_amount = order.market_meta.close * signal_strength.0;
        let order_size = dollar_amount / order.market_meta.close;

        match order.decision {
            // Entry
            Decision::Long => order.quantity = order_size,

            // Entry
            Decision::Short => order.quantity = -order_size,

            // Exit
            _ => order.quantity = 0.0 - position.as_ref().unwrap().quantity,
        }
    }
}
