use chrono::Utc;
use rotom_data::{
    assets::level::Level,
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use rotom_oms::model::{order::OrderEvent, ClientOrderId};
use rotom_strategy::Decision;

#[derive(Debug, Default)]
pub struct SpotArbBookData {
    pub best_bid: Level,
    pub best_ask: Level,
}

#[derive(Debug, Default)]
pub struct SpotArbOrderGenerator {
    pub liquid_exchange: SpotArbBookData,
    pub illiquid_exchange: SpotArbBookData,
}

impl SpotArbOrderGenerator {
    // fn keep_order_at_bba_illiquid(
    //     &mut self,
    //     exchange: &ExchangeId,
    //     instrument: &Instrument,
    //     book_data: &EventOrderBook,
    // ) -> Option<OrderEvent> {
    //     if exchange == &IlliquidExchange::CLIENT {
    //         if self.illiquid_exchange.best_bid != book_data.bids[0] {
    //             self.illiquid_exchange.best_bid = book_data.bids[0];
    //             return Some(OrderEvent {
    //                 order_request_time: Utc::now(),
    //                 exchange: IlliquidExchange::CLIENT,
    //                 instrument: instrument.clone(),
    //                 client_order_id: ClientOrderId::random(),
    //                 market_meta: MarketMeta {
    //                     close: self.illiquid_exchange.best_bid.price * 0.98,
    //                     time: Utc::now(),
    //                 },
    //                 decision: Decision::Long,
    //                 // original_quantity: self.illiquid_exchange.best_bid.size,
    //                 original_quantity: 1.0,
    //                 cumulative_quantity: 0.0,
    //                 order_kind: rotom_oms::model::OrderKind::Limit,
    //                 exchange_order_status: None,
    //                 internal_order_state: rotom_oms::model::order::OrderState::InTransit,
    //                 filled_gross: 0.0,
    //                 enter_avg_price: 0.0,
    //                 fees: 0.0,
    //                 last_execution_time: None,
    //             });
    //         }

    //         if self.illiquid_exchange.best_ask != book_data.asks[0] {
    //             return None;
    //             // self.illiquid_exchange.best_ask = book_data.asks[0];
    //             // return Some(OrderEvent {
    //             //     order_request_time: Utc::now(),
    //             //     exchange: IlliquidExchange::CLIENT,
    //             //     instrument: instrument.clone(),
    //             //     client_order_id: None,
    //             //     market_meta: MarketMeta {
    //             //         close: self.illiquid_exchange.best_ask.price * 0.98,
    //             //         time: Utc::now(),
    //             //     },
    //             //     decision: Decision::Short,
    //             //     original_quantity: self.illiquid_exchange.best_ask.size,
    //             //     original_quantity: 1.0,
    //             //     cumulative_quantity: 0.0,
    //             //     order_kind: rotom_oms::model::OrderKind::Limit,
    //             //     exchange_order_status: None,
    //             //     internal_order_state: rotom_oms::model::order::OrderState::InTransit,
    //             //     filled_gross: 0.0,
    //             //     enter_avg_price: 0.0,
    //             //     fees: 0.0,
    //             //     last_execution_time: None,
    //             // });
    //         }
    //     }

    //     None
    // }

    pub fn process_market_event(
        &mut self,
        market_event: &MarketEvent<DataKind>,
    ) -> Option<OrderEvent> {
        // match &market_event.event_data {
        //     DataKind::OrderBook(book_data) => {
        //         // Keep order at bba for Illiquid exchange
        //         let bba_illiquid_order = self.keep_order_at_bba_illiquid(
        //             &market_event.exchange,
        //             &market_event.instrument,
        //             book_data,
        //         );

        //         return bba_illiquid_order;
        //     }
        //     DataKind::Trade(_trade) => {}
        // }

        // None
        Some(OrderEvent {
            order_request_time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            instrument: Instrument::new("op", "usdt"),
            client_order_id: ClientOrderId::random(),
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
}
