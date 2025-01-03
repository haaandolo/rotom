use async_trait::async_trait;
use chrono::Utc;

use super::model::BinanceSpotBookUpdate;
use crate::assets::orderbook::OrderBook;
use crate::error::SocketError;
use crate::exchange::binance::public_http::binance_public_http_client::BinancePublicData;
use crate::exchange::binance::public_http::requests::ticker_info::Filter;
use crate::model::event_book::EventOrderBook;
use crate::shared::subscription_models::ExchangeId;
use crate::shared::subscription_models::Instrument;
use crate::transformer::book::{InstrumentOrderBook, OrderBookUpdater};

pub const HTTP_BOOK_L2_SNAPSHOT_URL_BINANCE_SPOT: &str = "https://api.binance.com/api/v3/depth";
pub const HTTP_TICKER_INFO_URL_BINANCE_SPOT: &str =
    "https://api.binance.us/api/v3/exchangeInfo?symbol=";

#[derive(Default, Debug)]
pub struct BinanceSpotBookUpdater {
    pub updates_processed: u64,
    pub last_update_id: u64,
    pub prev_last_update_id: u64,
}

impl BinanceSpotBookUpdater {
    pub fn new(last_update_id: u64) -> Self {
        Self {
            updates_processed: 0,
            prev_last_update_id: last_update_id,
            last_update_id,
        }
    }

    pub fn is_first_update(&self) -> bool {
        self.updates_processed == 0
    }

    pub fn validate_first_update(&self, update: &BinanceSpotBookUpdate) -> Result<(), SocketError> {
        let expected_next_id = self.last_update_id + 1;
        if update.first_update_id <= expected_next_id && update.last_update_id >= expected_next_id {
            Ok(())
        } else {
            Err(SocketError::InvalidSequence {
                symbol: update.symbol.clone(),
                prev_last_update_id: self.last_update_id,
                first_update_id: update.first_update_id,
            })
        }
    }

    pub fn validate_next_update(&self, update: &BinanceSpotBookUpdate) -> Result<(), SocketError> {
        let expected_next_id = self.last_update_id + 1;
        if update.first_update_id == expected_next_id {
            Ok(())
        } else {
            Err(SocketError::InvalidSequence {
                symbol: update.symbol.clone(),
                prev_last_update_id: self.last_update_id,
                first_update_id: update.first_update_id,
            })
        }
    }
}

#[async_trait]
impl OrderBookUpdater for BinanceSpotBookUpdater {
    type OrderBook = OrderBook;
    type UpdateEvent = BinanceSpotBookUpdate;

    async fn init(instrument: &Instrument) -> Result<InstrumentOrderBook<Self>, SocketError> {
        let snapshot = BinancePublicData::get_book_snapshot(instrument).await?;
        let ticker_info = BinancePublicData::get_ticker_info(instrument).await?;

        let tick_size = ticker_info.symbols[0].filters.iter().find_map(|filter| {
            if let Filter::PriceFilter { tick_size, .. } = filter {
                Some(tick_size.to_owned())
            } else {
                None
            }
        });

        match tick_size {
            Some(tick_size) => {
                let mut orderbook_init = OrderBook::new(tick_size);
                orderbook_init.process_lvl2(snapshot.bids, snapshot.asks);

                Ok(InstrumentOrderBook {
                    instrument: instrument.clone(),
                    updater: Self::new(snapshot.last_update_id),
                    book: orderbook_init,
                })
            }
            None => Err(SocketError::TickSizeError {
                base: instrument.base.clone(),
                quote: instrument.quote.clone(),
                exchange: ExchangeId::BinanceSpot,
            }),
        }
    }

    fn update(
        &mut self,
        book: &mut Self::OrderBook,
        update: Self::UpdateEvent,
    ) -> Result<Option<EventOrderBook>, SocketError> {
        if update.last_update_id <= self.last_update_id {
            return Ok(None);
        }

        if self.is_first_update() {
            self.validate_first_update(&update)?;
        } else {
            self.validate_next_update(&update)?;
        }

        book.last_update_time = Utc::now();
        book.process_lvl2(update.bids, update.asks);

        self.updates_processed += 1;
        self.prev_last_update_id = self.last_update_id;
        self.last_update_id = update.last_update_id;

        Ok(Some(book.book_snapshot()))
    }
}

/*----- */
// How to manage local orderbook - Binance Spot
/*----- */
// 1. Open a stream to wss://stream.binance.com:9443/ws/BTCUSDT@depth.
// 2. Buffer the events you receive from the stream.
// 3. Get a depth snapshot from <https://api.binance.com/api/v3/depth?symbol=BNBBTC&limit=1000>.
// 4. -- *DIFFERENT FROM FUTURES* --
//    Drop any event where u is <= lastUpdateId in the snapshot.
// 5. -- *DIFFERENT FROM FUTURES* --
//    The first processed event should have U <= lastUpdateId+1 AND u >= lastUpdateId+1.
// 6. -- *DIFFERENT FROM FUTURES* --
//    While listening to the stream, each new event's U should be equal to the
//    previous event's u+1, otherwise initialize the process from step 3.
// 7. The data in each event is the absolute quantity for a price level.
// 8. If the quantity is 0, remove the price level.
//
// Notes:
//  - Receiving an event that removes a price level that is not in your local order book can happen and is normal.
//  - Uppercase U => first_update_id
//  - Lowercase u => last_update_id,
//
// See docs: https://binance-docs.github.io/apidocs/spot/en/#how-to-manage-a-local-order-book-correctly
