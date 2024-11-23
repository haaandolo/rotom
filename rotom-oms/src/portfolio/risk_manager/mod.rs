pub mod default_risk_manager;
pub mod spot_cross_exchange_arb_rm;

use crate::model::{order::{Order, RequestOpen}, OrderKind};

/*----- */
// Order Evaluator
/*----- */
pub trait OrderEvaluator {
    const DEFAULT_ORDER_TYPE: OrderKind;

    fn evaluate_order(&self, order: Order<RequestOpen>) -> Option<Order<RequestOpen>>;
}
