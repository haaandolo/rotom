use async_trait::async_trait;
use chrono::Utc;

use crate::{
    assets::orderbook::OrderBook,
    error::SocketError,
    exchange::{phemex::PhemexSpotPublicData, PublicHttpConnector},
    model::event_book::EventOrderBook,
    shared::subscription_models::{ExchangeId, Instrument},
    transformer::book::{InstrumentOrderBook, OrderBookUpdater},
    AssetFormatted,
};

use super::model::PhemexOrderBookUpdate;

#[derive(Default, Debug)]
pub struct PhemexSpotBookUpdater {
    pub prev_last_update_id: u64,
}

impl PhemexSpotBookUpdater {
    pub fn validate_next_update(&self, update: &PhemexOrderBookUpdate) -> Result<(), SocketError> {
        if update.sequence > self.prev_last_update_id {
            Ok(())
        } else {
            Err(SocketError::InvalidSequence {
                symbol: update.symbol.clone(),
                prev_last_update_id: self.prev_last_update_id,
                first_update_id: update.sequence,
            })
        }
    }
}

// Note: I don't think Phemex's sequencing is relevant - maybe only the fact that the current one is greater
// than the last one. Phemex does however send a snapshot of the full order if there is something wrong. So,
// use this as a resetting point
#[async_trait]
impl OrderBookUpdater for PhemexSpotBookUpdater {
    type OrderBook = OrderBook;
    type UpdateEvent = PhemexOrderBookUpdate;

    async fn init(instrument: &Instrument) -> Result<InstrumentOrderBook<Self>, SocketError> {
        let ticker_info = PhemexSpotPublicData::get_ticker_info(instrument.clone()).await?;
        let ticker_formatted = AssetFormatted::from((&ExchangeId::PhemexSpot, instrument));

        let tick_size = ticker_info.data.products.into_iter().find_map(|t| {
            if t.symbol == ticker_formatted.0 {
                t.tick_size
            } else {
                None
            }
        });

        match tick_size {
            Some(tick_size) => {
                let orderbook_init = OrderBook::new(tick_size);

                Ok(InstrumentOrderBook {
                    instrument: instrument.clone(),
                    updater: Self::default(),
                    book: orderbook_init,
                })
            }
            None => Err(SocketError::TickSizeError {
                base: instrument.base.clone(),
                quote: instrument.quote.clone(),
                exchange: ExchangeId::PhemexSpot,
            }),
        }
    }

    fn update(
        &mut self,
        book: &mut Self::OrderBook,
        update: Self::UpdateEvent,
    ) -> Result<Option<EventOrderBook>, SocketError> {
        if update.message_type == "snapshot" {
            book.reset();
            book.process_lvl2(update.book.bids, update.book.asks);
            self.prev_last_update_id = update.sequence;
        } else {
            self.validate_next_update(&update)?;

            book.last_update_time = Utc::now();
            book.process_lvl2(update.book.bids, update.book.asks);

            self.prev_last_update_id = update.sequence;
        }

        Ok(Some(book.book_snapshot()))
    }
}
