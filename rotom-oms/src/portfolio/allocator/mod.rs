pub mod default_allocator;

use rotom_strategy::SignalStrength;

use super::{position::Position, OrderEvent};

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
