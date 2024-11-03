use serde::Deserialize;

use crate::portfolio::position::Position;

use super::{OrderAllocator, OrderEvent};
use rotom_strategy::{Decision, SignalStrength};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default, Deserialize)]
pub struct DefaultAllocator {
    pub default_order_value: f64,
}

impl OrderAllocator for DefaultAllocator {
    fn allocate_order(
        &self,
        order: &mut OrderEvent,
        position: Option<&Position>,
        signal_strength: SignalStrength,
    ) {
        // Calculate exact order_size
        let default_order_size = self.default_order_value / order.market_meta.close;
        println!("default order 1: {}", default_order_size);

        // Then round it to a more appropriate decimal place
        let default_order_size = (default_order_size * 10000.0).floor() / 10000.0;
        println!("default order 2: {}", default_order_size);

        match order.decision {
            // Entry
            Decision::Long => order.quantity = default_order_size * signal_strength.0,

            // Entry
            Decision::Short => order.quantity = -default_order_size * signal_strength.0,

            // Exit
            _ => order.quantity = 0.0 - position.as_ref().unwrap().quantity,
        }
    }
}
