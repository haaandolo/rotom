use async_trait::async_trait;
use chrono::Utc;
use std::mem;

use crate::data::{
    error::SocketError,
    model::{
        book::EventOrderBook,
        event::MarketEvent,
        subs::{ExchangeId, Instrument, StreamType},
    },
    shared::{orderbook::OrderBook, utils::current_timestamp_utc},
    transformer::book::{InstrumentOrderBook, OrderBookUpdater},
};

use super::model::PoloniexSpotBookUpdate;

#[derive(Default, Debug)]
pub struct PoloniexSpotBookUpdater {
    pub prev_last_update_id: u64,
}

impl PoloniexSpotBookUpdater {
    pub fn validate_next_update(&self, current_update_id: u64) -> Result<(), SocketError> {
        let expected_next_id = self.prev_last_update_id + 1;
        if current_update_id == expected_next_id {
            Ok(())
        } else {
            Err(SocketError::InvalidSequence {
                prev_last_update_id: self.prev_last_update_id,
                first_update_id: current_update_id,
            })
            // Err(SocketError::OrderBookNonTerminal {
            //     message: String::from("Sequence broken for Poloniex Spot L2"),
            // })
        }
    }
}

#[async_trait]
impl OrderBookUpdater for PoloniexSpotBookUpdater {
    type OrderBook = OrderBook;
    type UpdateEvent = PoloniexSpotBookUpdate;

    async fn init(instrument: &Instrument) -> Result<InstrumentOrderBook<Self>, SocketError> {
        let orderbook_init = OrderBook::new(0.001);
        Ok(InstrumentOrderBook {
            instrument: instrument.clone(),
            updater: Self::default(),
            book: orderbook_init,
        })
    }

    fn update(
        &mut self,
        book: &mut Self::OrderBook,
        mut update: Self::UpdateEvent,
    ) -> Result<Option<MarketEvent<EventOrderBook>>, SocketError> {
        let update_data = mem::take(&mut update.data[0]); // TODO
        if update.action == "snapshot" {
            book.reset();
            book.process_lvl2(update_data.bids, update_data.asks);
            self.prev_last_update_id = update_data.id;
            Ok(None)
        } else {
            self.validate_next_update(update_data.id)?;

            book.last_update_time = Utc::now();
            book.process_lvl2(update_data.bids, update_data.asks);

            self.prev_last_update_id = update_data.id;

            let book_snapshot = book.book_snapshot();
            Ok(Some(MarketEvent {
                exchange_time: update_data.timestamp,
                received_time: current_timestamp_utc(),
                seq: update_data.id,
                exchange: ExchangeId::PoloniexSpot,
                stream_type: StreamType::L2,
                symbol: update_data.symbol,
                event_data: book_snapshot,
            }))
        }
    }
}

/*----- */
// How to manage local orderbook - Poloniex Spot
/*----- */
// 1. Send a book_lv2 subscription message.
// 2. Receive a snapshot message from the server.
// 3. Use an appropriate data structure to store the received book.
// 4. Receive an incremental order book message (update) from the server and make changes depending on [price, quantity] pair data:
//    - When quantity is positive, update the corresponding price of your order book with this quantity.
//    - When quantity is 0, delete this price from your order book.
// 5. Receive an order book message (snapshot) from the server, reset your order book data structure to match this new order book.
//
// Notes:
// If id of the last message does not match lastId of the current message then the client has lost connection with the server and must re-subscribe to the channel.
// See docs: https://api-docs.poloniex.com/spot/websocket/market-data
