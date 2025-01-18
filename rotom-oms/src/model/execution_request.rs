use rotom_data::shared::subscription_models::Instrument;
use rotom_strategy::Decision;

use crate::execution_manager::builder::TraderId;

use super::{ClientOrderId, OrderKind};

/*----- */
// Open Order
/*----- */
#[derive(Debug, Clone)]
pub struct OpenOrder {
    pub trader_id: TraderId,
    pub client_order_id: ClientOrderId,
    // Used for market or limit orders
    pub price: f64,
    // Used for market or limit orders
    pub quantity: f64,
    // Used for market orders
    pub notional_amount: f64,
    pub decision: Decision,
    pub order_kind: OrderKind,
    pub instrument: Instrument,
}

/*----- */
// Cancel Order
/*----- */
#[derive(Debug)]
pub struct CancelOrder {
    pub trader_id: TraderId,
    // Used to identify order at the given exchange
    pub client_order_id: String, // smol str,
    pub symbol: String,          // smol str
}

/*----- */
// Wallet transfer
/*----- */
#[derive(Debug, Clone)]
pub struct WalletTransfer {
    pub trader_id: TraderId,
    pub coin: String,            // smol
    pub wallet_address: String,  // can be static str todo: change to deposit address
    pub network: Option<String>, // smol, probs can be static str
    pub amount: f64,
}

/*----- */
// Execution Requests
/*----- */
#[derive(Debug)]
pub enum ExecutionRequest {
    Open(OpenOrder),
    Cancel(CancelOrder),
    CancelAll(CancelOrder),
    Transfer(WalletTransfer),
}

impl std::fmt::Display for ExecutionRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionRequest::Open(request) => write!(f, "{:#?}", request),
            ExecutionRequest::Cancel(request) => write!(f, "{:#?}", request),
            ExecutionRequest::CancelAll(request) => write!(f, "{:#?}", request),
            ExecutionRequest::Transfer(request) => write!(f, "{:#?}", request),
        }
    }
}

// impl From<(&TickerPrecision, &OrderEvent)> for OpenOrder {
//     fn from((precision, order): (&TickerPrecision, &OrderEvent)) -> Self {
//         Self {
//             price: round_float_to_precision(order.market_meta.close, precision.price_precision),
//             quantity: round_float_to_precision(
//                 order.original_quantity,
//                 precision.quantity_precision,
//             ),
//             // todo: cum quantity * avg price?
//             notional_amount: round_float_to_precision(
//                 order.cumulative_quantity * order.enter_avg_price,
//                 precision.notional_precision,
//             ),
//             decision: order.decision,
//             order_kind: order.order_kind.clone(),
//             instrument: order.instrument.clone(),
//         }
//     }
// }
