pub mod default_risk_manager;

use super::{OrderEvent, OrderType};

/*----- */
// Order Evaluator
/*----- */
pub trait OrderEvaluator {
    const DEFAULT_ORDER_TYPE: OrderType;

    fn evaluate_order(&self, order: OrderEvent) -> Option<OrderEvent>;
}
