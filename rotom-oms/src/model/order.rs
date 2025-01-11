use chrono::{DateTime, Utc};
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::execution_manager::builder::TraderId;

use super::{
    account_data::{AccountDataOrder, ExecutionResponse, OrderStatus},
    ClientOrderId, OrderKind,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub enum OrderFill {
    Full,
    Partial,
}

/*----- */
// OrderEvent
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct OrderEvent {
    pub order_request_time: DateTime<Utc>, // before
    pub exchange: ExchangeId,              // before
    pub client_order_id: ClientOrderId,    // before
    pub instrument: Instrument,            // before
    // Metadata propagated from source MarketEvent
    pub market_meta: MarketMeta, // before
    // LONG, CloseLong, SHORT or CloseShort
    pub decision: Decision, // before
    // Original / desired quantity of order
    pub original_quantity: f64, // before
    // Cumulative filled quantity
    pub cumulative_quantity: f64, // after
    // MARKET, LIMIT etc
    pub order_kind: OrderKind,                      // before
    pub exchange_order_status: Option<OrderStatus>, // after
    pub internal_order_state: OrderState,           // before
    // Filled gross amount of quote currency
    pub filled_gross: f64, // after
    // Enter average price excluding fees. Position.filled_gross / Position.cumulative_quantity
    pub enter_avg_price: f64, // after
    // Cumulative value: todo is it base or quote asset?
    pub fees: f64, // after
    // Orders last executed time
    pub last_execution_time: Option<DateTime<Utc>>,
}

impl OrderEvent {
    pub fn get_dollar_value(&self) -> f64 {
        self.original_quantity * self.market_meta.close
    }

    pub fn get_exchange(&self) -> ExchangeId {
        self.exchange
    }

    pub fn calculate_avg_price(&self) -> f64 {
        self.filled_gross / self.cumulative_quantity
    }

    pub fn set_state(&mut self, state: OrderState) {
        self.internal_order_state = state
    }

    pub fn is_order_filled(&self) -> bool {
        if let Some(order_status) = self.exchange_order_status {
            order_status == OrderStatus::Filled
        } else {
            false
        }
    }

    pub fn update_order_from_account_data_stream(
        &mut self,
        account_data_update: &AccountDataOrder,
    ) {
        self.set_state(OrderState::Open);
        self.exchange_order_status = Some(account_data_update.status);
        self.filled_gross = account_data_update.filled_gross; // Filled_gross field in AccountDataOrder is cumulative so we can just set it each time
        self.cumulative_quantity += account_data_update.quantity; // Quantity field in AccountDataOrder is not cumulative we have to "+=" here
        self.enter_avg_price = self.calculate_avg_price(); // This step has to happen after the cumulative_quantity & filled_gross gets updated
        self.fees += account_data_update.fee;
        self.last_execution_time = Some(account_data_update.execution_time);
        self.client_order_id = ClientOrderId(account_data_update.client_order_id.clone());
    }

    // If the state is not open or intransit, we can set a ClientOrderId
    pub fn set_client_id(&mut self, order_update: &AccountDataOrder) {
        match self.internal_order_state.is_order_open() {
            false => self.client_order_id = ClientOrderId(order_update.client_order_id.clone()),
            true => (),
        }
    }

    pub fn is_order_complete(&self) -> bool {
        if self.internal_order_state == OrderState::Complete {
            return true;
        }
        false
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum OrderState {
    Open,
    InTransit,
    Complete,
    RequestOpen,
    Cancelled,
}

impl OrderState {
    pub fn is_order_open(&self) -> bool {
        match self {
            OrderState::Open => true,
            OrderState::InTransit => true,
            OrderState::Complete => false,
            OrderState::RequestOpen => false,
            OrderState::Cancelled => false,
        }
    }
}

// todo: is this required??
impl From<OrderStatus> for OrderState {
    fn from(order_status: OrderStatus) -> Self {
        match order_status {
            OrderStatus::New => OrderState::Open,
            OrderStatus::Canceled => OrderState::Cancelled,
            OrderStatus::Rejected => OrderState::Cancelled,
            OrderStatus::Expired => OrderState::Cancelled,
            OrderStatus::PendingNew => OrderState::Open,
            OrderStatus::PartiallyFilled => OrderState::Open,
            OrderStatus::Filled => OrderState::Complete,
            OrderStatus::Trade => OrderState::Open,
            OrderStatus::PendingCancel => OrderState::Cancelled,
            OrderStatus::ExpiredInMatch => OrderState::Cancelled,
            OrderStatus::PartiallyCanceled => OrderState::Open,
            OrderStatus::Failed => OrderState::Cancelled,
        }
    }
}

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
// Execution Manager Subscription Request
/*----- */
#[derive(Debug, Clone)]
pub struct ExecutionManagerSubscribe {
    pub trader_id: TraderId,
    pub execution_response_tx: mpsc::UnboundedSender<ExecutionResponse>,
}

/*----- */
// Execution Requests
/*----- */
#[derive(Debug)]
pub enum ExecutionRequest {
    Subscribe(ExecutionManagerSubscribe),
    Open(OpenOrder),
    Cancel(CancelOrder),
    CancelAll(CancelOrder),
    Transfer(WalletTransfer),
}

impl std::fmt::Display for ExecutionRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionRequest::Subscribe(request) => write!(f, "{:#?}", request),
            ExecutionRequest::Open(request) => write!(f, "{:#?}", request),
            ExecutionRequest::Cancel(request) => write!(f, "{:#?}", request),
            ExecutionRequest::CancelAll(request) => write!(f, "{:#?}", request),
            ExecutionRequest::Transfer(request) => write!(f, "{:#?}", request),
        }
    }
}
