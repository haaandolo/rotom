use async_trait::async_trait;
use chrono::Utc;
use std::mem;

use super::model::{PoloniexSpotBookData, PoloniexSpotBookUpdate, PoloniexSpotTickerInfo};
use crate::{
    assets::orderbook::OrderBook,
    error::SocketError,
    event_models::event_book::EventOrderBook,
    shared::{subscription_models::Instrument, utils::decimal_places_to_number},
    transformer::book::{InstrumentOrderBook, OrderBookUpdater},
};

pub const HTTP_TICKER_INFO_URL_POLONIEX_SPOT: &str = "https://api.poloniex.com/markets/";

#[derive(Default, Debug)]
pub struct PoloniexSpotBookUpdater {
    pub prev_last_update_id: u64,
}

impl PoloniexSpotBookUpdater {
    pub fn validate_next_update(&self, update: &PoloniexSpotBookData) -> Result<(), SocketError> {
        let expected_next_id = self.prev_last_update_id + 1;
        if update.id == expected_next_id {
            Ok(())
        } else {
            Err(SocketError::InvalidSequence {
                symbol: update.symbol.clone(),
                prev_last_update_id: self.prev_last_update_id,
                first_update_id: update.id,
            })
        }
    }
}

#[async_trait]
impl OrderBookUpdater for PoloniexSpotBookUpdater {
    type OrderBook = OrderBook;
    type UpdateEvent = PoloniexSpotBookUpdate;

    async fn init(instrument: &Instrument) -> Result<InstrumentOrderBook<Self>, SocketError> {
        let ticker_info_url = format!(
            "{}{}_{}",
            HTTP_TICKER_INFO_URL_POLONIEX_SPOT,
            instrument.base.to_uppercase(),
            instrument.quote.to_uppercase()
        );

        let ticker_info = reqwest::get(ticker_info_url)
            .await
            .map_err(SocketError::Http)?
            .json::<Vec<PoloniexSpotTickerInfo>>()
            .await
            .map_err(SocketError::Http)?;

        let price_scale = ticker_info[0].symbol_trade_limit.quantity_scale;
        let tick_size = decimal_places_to_number(price_scale);
        let orderbook_init = OrderBook::new(tick_size);

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
    ) -> Result<Option<EventOrderBook>, SocketError> {
        let update_data = mem::take(&mut update.data[0]); // TODO
        if update.action == "snapshot" {
            book.reset();
            book.process_lvl2(update_data.bids, update_data.asks);
            self.prev_last_update_id = update_data.id;
            Ok(None)
        } else {
            self.validate_next_update(&update_data)?;

            book.last_update_time = Utc::now();
            book.process_lvl2(update_data.bids, update_data.asks);

            self.prev_last_update_id = update_data.id;

            Ok(Some(book.book_snapshot()))
        }
    }
}

/*----- */
// How to manage local orderbook - Poloniex Spot
/*----- */
// 1. Send a book_lv2 subscription message.
// 2. Receive a snapshot message from the server.
// 3. Use an appropriate data structure to store the received book.
// 4, Receive an incremental order book message (update) from the server and make changes depending on
//    - When quantity is positive, update the corresponding price of your order book with this quantity.
//    - When quantity is 0, delete this price from your order book.
// 5. Receive an order book message (snapshot) from the server, reset your order book data structure to match this new order book.
//
// Notes:
// If id of the last message does not match lastId of the current message then the client has lost connection with the server and must re-subscribe to the channel.
// See docs: https://api-docs.poloniex.com/spot/websocket/market-data
//
// Price scale:
// priceScale is referring to the max number of decimals allowed for a given symbol. For example, priceScale of 2 implies accepted values like 200.34, 13.2, 100 and priceScale of 0 implies accepted values like 200, 92, 13.
// Source: https://api-docs.poloniex.com/spot/api/public/reference-data
