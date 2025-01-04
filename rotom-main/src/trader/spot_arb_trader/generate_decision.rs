use chrono::Utc;
use rotom_data::{
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use rotom_oms::model::order::OrderEvent;
use rotom_strategy::Decision;

pub fn process_market_event(_market_event: &MarketEvent<DataKind>) -> Option<OrderEvent> {
    Some(OrderEvent {
        order_request_time: Utc::now(),
        exchange: ExchangeId::BinanceSpot,
        instrument: Instrument::new("op", "usdt"),
        client_order_id: None,
        market_meta: MarketMeta {
            close: 1.0,
            time: Utc::now(),
        },
        decision: Decision::Long,
        original_quantity: 1.0,
        cumulative_quantity: 0.0,
        order_kind: rotom_oms::model::OrderKind::Limit,
        exchange_order_status: None,
        internal_order_state: rotom_oms::model::order::OrderState::InTransit,
        filled_gross: 0.0,
        enter_avg_price: 0.0,
        fees: 0.0,
        last_execution_time: None,
    })
}
