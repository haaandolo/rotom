pub mod default_risk_manager;
pub mod spot_cross_exchange_arb_rm;

use crate::model::{order::OrderEvent, OrderKind};

/*----- */
// Order Evaluator
/*----- */
pub trait OrderEvaluator {
    const DEFAULT_ORDER_TYPE: OrderKind;

    fn evaluate_order(&self, order: OrderEvent) -> Option<OrderEvent>;
}
