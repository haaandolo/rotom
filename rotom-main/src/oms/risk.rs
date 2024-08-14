use super::{OrderEvent, OrderType};

/*----- */
// Order Evaluator
/*----- */
pub trait OrderEvaluator {
    const DEFAULT_ORDER_TYPE: OrderType;

    fn evaluate_order(&self, order: OrderEvent) -> Option<OrderEvent>;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct DefaultRisk {}

impl OrderEvaluator for DefaultRisk {
    const DEFAULT_ORDER_TYPE: OrderType = OrderType::Market;

    fn evaluate_order(&self, mut order: OrderEvent) -> Option<OrderEvent> {
        if self.risk_too_high(&order) {
            return None;
        }
        order.order_type = DefaultRisk::DEFAULT_ORDER_TYPE;
        Some(order)
    }
}

impl DefaultRisk {
    fn risk_too_high(&self, _: &OrderEvent) -> bool {
        false
    }
}