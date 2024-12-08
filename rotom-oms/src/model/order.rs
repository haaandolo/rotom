use chrono::{DateTime, Utc};
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};

use super::{
    account_data::{AccountDataOrder, OrderStatus},
    ClientOrderId, OrderKind,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub enum OrderFill {
    Full,
    Partial,
}

#[derive(Debug)]
pub struct AssetFormatted(pub String);

/*----- */
// OrderEvent
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct OrderEvent {
    pub time: DateTime<Utc>,
    pub exchange: ExchangeId,
    pub client_order_id: Option<ClientOrderId>,
    pub instrument: Instrument,
    // Metadata propagated from source MarketEvent
    pub market_meta: MarketMeta,
    // LONG, CloseLong, SHORT or CloseShort
    pub decision: Decision,
    // +ve or -ve Quantity depending on Decision
    pub quantity: f64,
    // MARKET, LIMIT etc
    pub order_kind: OrderKind,
    pub order_status: Option<OrderStatus>,
    pub state: OrderState,
    // Filled gross of quote currency
    pub filled_gross: f64,
}

impl OrderEvent {
    pub fn get_dollar_value(&self) -> f64 {
        self.quantity * self.market_meta.close
    }

    // If the state is not open or intransit, we can set a ClientOrderId
    pub fn set_client_id(&mut self, order_update: &AccountDataOrder) {
        match self.state.is_order_open() {
            false => {
                self.client_order_id = Some(ClientOrderId(order_update.client_order_id.clone()))
            }
            true => (),
        }
    }

    pub fn update_order(&mut self, order_update: &AccountDataOrder) {
        self.filled_gross = order_update.filled_gross
    }
}

impl From<(&ExchangeId, &Instrument)> for AssetFormatted {
    fn from((exchange, instrument): (&ExchangeId, &Instrument)) -> Self {
        match exchange {
            ExchangeId::BinanceSpot => {
                AssetFormatted(format!("{}{}", instrument.base, instrument.quote).to_uppercase())
            }
            ExchangeId::PoloniexSpot => {
                AssetFormatted(format!("{}_{}", instrument.base, instrument.quote).to_uppercase())
            }
        }
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

/*----- */
// Open Order
/*----- */
#[derive(Debug)]
pub struct OpenOrder {
    pub price: f64,
    pub quantity: f64,
    pub decision: Decision,
    pub order_kind: OrderKind,
    pub instrument: Instrument,
}

impl From<&OrderEvent> for OpenOrder {
    fn from(order: &OrderEvent) -> Self {
        Self {
            price: order.market_meta.close,
            quantity: order.quantity,
            decision: order.decision,
            order_kind: order.order_kind.clone(),
            instrument: order.instrument.clone(),
        }
    }
}

/*----- */
// Cancel Order
/*----- */
#[derive(Debug)]
pub struct CancelOrder {
    pub id: String,     // smol str,
    pub symbol: String, // smol str
}

impl From<&OrderEvent> for CancelOrder {
    fn from(order: &OrderEvent) -> Self {
        Self {
            id: order.client_order_id.clone().unwrap().0,
            symbol: AssetFormatted::from((&order.exchange, &order.instrument)).0,
        }
    }
}

/*----- */
// Wallet transfer
/*----- */
#[derive(Debug)]
pub struct WalletTransfer {
    pub coin: String,
    pub wallet_address: String,
    pub network: Option<String>,
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
    Transfer(WalletTransfer)
}