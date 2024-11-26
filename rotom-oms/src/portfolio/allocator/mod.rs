pub mod default_allocator;
pub mod spot_arb_allocator;

use rotom_strategy::SignalStrength;

use crate::model::order::OrderEvent;

use super::position::Position;

/*----- */
// Order Allocator
/*----- */
pub trait OrderAllocator {
    fn allocate_order(
        &self,
        order: &mut OrderEvent,
        position: Option<&Position>,
        signal_strength: SignalStrength,
    );
}
