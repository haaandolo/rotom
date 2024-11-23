pub mod default_risk_manager;
pub mod spot_cross_exchange_arb_rm;

use crate::model::order::{Order, RequestOpen};

use super::OrderType;

/*----- */
// Order Evaluator
/*----- */
pub trait OrderEvaluator {
    const DEFAULT_ORDER_TYPE: OrderType;

    fn evaluate_order(&self, order: Order<RequestOpen>) -> Option<Order<RequestOpen>>;
}
