use chrono::{DateTime, Utc};
use rotom_data::{
    model::ticker_info::TickerSpecs,
    shared::{
        subscription_models::{ExchangeId, Instrument},
        utils::rtp,
    },
};
use rotom_strategy::Decision;

use crate::execution_manager::builder::TraderId;

use super::{ClientOrderId, OrderKind};

/*----- */
// ExecutionRequest
/*----- */
#[derive(Debug, Clone)]
pub enum ExecutionRequest {
    Open(Order<OpenOrder>),
    Cancel(Order<CancelOrder>),
    CancelAll(Order<CancelOrder>),
    Transfer(Order<WalletTransfer>),
}

impl ExecutionRequest {
    pub fn get_exchange_id(&self) -> ExchangeId {
        match self {
            ExecutionRequest::Open(request) => request.exchange,
            ExecutionRequest::Cancel(request) => request.exchange,
            ExecutionRequest::CancelAll(request) => request.exchange,
            ExecutionRequest::Transfer(request) => request.exchange,
        }
    }
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

/*----- */
// Generic order type
/*----- */
#[derive(Debug, Clone)]
pub struct Order<Request> {
    pub trader_id: TraderId,
    pub exchange: ExchangeId,
    pub cid: ClientOrderId,
    pub requested_time: DateTime<Utc>,
    pub request: Request,
}

/*----- */
// Requests
/*----- */
#[derive(Debug, Clone)]
pub struct OpenOrder {
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

impl OpenOrder {
    pub fn sanitise(&mut self, specs: &TickerSpecs) {
        self.price = rtp(self.price, specs.price_precision);
        self.quantity = rtp(self.quantity, specs.quantity_precision);
        self.notional_amount = rtp(self.notional_amount, specs.notional_precision);
    }
}

#[derive(Debug, Clone)]
pub struct CancelOrder {
    pub symbol: String, // smol str
}

#[derive(Debug, Clone)]
pub struct WalletTransfer {
    pub coin: String,            // smol
    pub wallet_address: String,  // can be static str todo: change to deposit address
    pub network: Option<String>, // smol, probs can be static str
    pub amount: f64,
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
