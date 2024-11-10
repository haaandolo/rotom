pub mod default_risk_manager;
pub mod spot_cross_exchange_arb_rm;

use super::{OrderEvent, OrderType};

/*----- */
// Order Evaluator
/*----- */
pub trait OrderEvaluator {
    const DEFAULT_ORDER_TYPE: OrderType;

    fn evaluate_order(&self, order: OrderEvent) -> Option<OrderEvent>;
}
