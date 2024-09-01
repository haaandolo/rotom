pub mod engine;
pub mod strategy;
pub mod event;
pub mod execution;
pub mod oms;
pub mod statistic;
pub mod trader;

pub mod test_util {
    use crate::{
        execution::{Fees, FillEvent},
        oms::{position::{Position, Side}, OrderEvent, OrderType},
        strategy::{Decision, Signal},
    };
    use chrono::Utc;
    use rotom_data::{assets::level::Level, event_models::{event_trade::EventTrade, market_event::{DataKind, MarketEvent}}, shared::subscription_models::{ExchangeId, Instrument}, MarketMeta};

    // Build a [`MarketEvent`] of [`DataKind::PublicTrade`](DataKind) with the provided [`Side`].
    pub fn market_event_trade() -> MarketEvent<DataKind> {
        MarketEvent {
            exchange_time: Utc::now(),
            received_time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            instrument: Instrument::new("btc", "usdt"),
            event_data: DataKind::Trade(EventTrade {
                trade: Level::new(1000.0, 1.0),
                is_buy: true,
            }),
        }
    }

    // Build a [`Signal`].
    pub fn signal() -> Signal {
        Signal {
            time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            instrument: Instrument::new("btc", "usdt"),
            signals: Default::default(),
            market_meta: Default::default(),
        }
    }

    /// Build an [`OrderEvent`] to buy 1.0 contract.
    pub fn order_event() -> OrderEvent {
        OrderEvent {
            time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            instrument: Instrument::new("btc", "usdt"),
            market_meta: MarketMeta::default(),
            decision: Decision::default(),
            quantity: 1.0,
            order_type: OrderType::default(),
        }
    }

    /// Build a [`FillEvent`] for a single bought contract.
    pub fn fill_event() -> FillEvent {
        FillEvent {
            time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            instrument: Instrument::new("btc", "usdt"),
            market_meta: Default::default(),
            decision: Decision::default(),
            quantity: 1.0,
            fill_value_gross: 100.0,
            fees: Fees::default(),
        }
    }

    /// Build a [`Position`].
    pub fn position() -> Position {
        Position {
            position_id: "engine_id_trader_{}_{}_position".to_owned(),
            exchange: ExchangeId::BinanceSpot,
            instrument: Instrument::new("btc", "usdt"),
            meta: Default::default(),
            side: Side::Buy,
            quantity: 1.0,
            enter_fees: Default::default(),
            enter_fees_total: 0.0,
            enter_avg_price_gross: 100.0,
            enter_value_gross: 100.0,
            exit_fees: Default::default(),
            exit_fees_total: 0.0,
            exit_avg_price_gross: 0.0,
            exit_value_gross: 0.0,
            current_symbol_price: 100.0,
            current_value_gross: 100.0,
            unrealised_profit_loss: 0.0,
            realised_profit_loss: 0.0,
        }
    }
}