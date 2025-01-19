use chrono::{DateTime, Utc};
use rotom_data::{
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};

use super::{
    execution_response::{OrderResponse, OrderStatus},
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

    pub fn update_order_from_account_data_stream(&mut self, account_data_update: &OrderResponse) {
        self.set_state(OrderState::Open);
        self.exchange_order_status = Some(account_data_update.status);
        self.filled_gross = account_data_update.cumulative_quote; // Filled_gross field in OrderResponse is cumulative so we can just set it each time
        self.cumulative_quantity += account_data_update.current_executed_quantity; // Quantity field in OrderResponse is not cumulative we have to "+=" here
        self.enter_avg_price = self.calculate_avg_price(); // This step has to happen after the cumulative_quantity & filled_gross gets updated
        self.fees += account_data_update.fee;
        self.last_execution_time = Some(account_data_update.execution_time);
        self.client_order_id = ClientOrderId(account_data_update.client_order_id.clone());
    }

    // If the state is not open or intransit, we can set a ClientOrderId
    pub fn set_client_id(&mut self, order_update: &OrderResponse) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, Debug, PartialEq)]
    struct CoinId {
        id: String,
    }

    #[test]
    fn test_partial_deserialization() {
        // Test case 1: Basic JSON with extra field
        let json = r#"
            {
                "id": "coin123",
                "balance": 1000
            }
        "#;

        let coin = serde_json::from_str::<CoinId>(json).unwrap();
        assert_eq!(
            coin,
            CoinId {
                id: String::from("coin123")
            }
        );

        // Test case 2: JSON with multiple extra fields
        let complex_json = r#"
            {
                "id": "coin456",
                "balance": 2000,
                "owner": "alice",
                "created_at": "2024-01-16",
                "active": true
            }
        "#;

        let coin = serde_json::from_str::<CoinId>(complex_json).unwrap();
        assert_eq!(
            coin,
            CoinId {
                id: String::from("coin456")
            }
        );

        // Test case 3: JSON with different field order
        let reordered_json = r#"
            {
                "balance": 3000,
                "id": "coin789"
            }
        "#;

        let coin = serde_json::from_str::<CoinId>(reordered_json).unwrap();
        assert_eq!(
            coin,
            CoinId {
                id: String::from("coin789")
            }
        );
    }
}
