use async_trait::async_trait;
use chrono::Utc;
// use futures::try_join;

use crate::{
    assets::orderbook::OrderBook,
    error::SocketError,
    exchange::{ascendex::AscendExSpotPublicData, PublicHttpConnector},
    model::event_book::EventOrderBook,
    shared::subscription_models::{ExchangeId, Instrument},
    transformer::book::{InstrumentOrderBook, OrderBookUpdater},
    AssetFormatted,
};

use super::model::AscendExBookUpdate;

#[derive(Default, Debug)]
pub struct AscendExSpotBookUpdater {
    pub updates_processed: u64,
    pub sequence_number: u64,
}

impl AscendExSpotBookUpdater {
    pub fn new(sequence_number: u64) -> Self {
        Self {
            updates_processed: 0,
            sequence_number,
        }
    }

    pub fn is_first_update(&self) -> bool {
        self.updates_processed == 0
    }

    pub fn validate_first_update(&self, update: &AscendExBookUpdate) -> Result<(), SocketError> {
        if update.data.seqnum > self.sequence_number {
            Ok(())
        } else {
            Err(SocketError::InvalidSequence {
                symbol: update.symbol.clone(),
                prev_last_update_id: self.sequence_number,
                first_update_id: update.data.seqnum,
            })
        }
    }

    pub fn validate_next_update(&self, update: &AscendExBookUpdate) -> Result<(), SocketError> {
        if update.data.seqnum == self.sequence_number + 1 {
            Ok(())
        } else {
            Err(SocketError::InvalidSequence {
                symbol: update.symbol.clone(),
                prev_last_update_id: self.sequence_number,
                first_update_id: update.data.seqnum,
            })
        }
    }
}

#[async_trait]
impl OrderBookUpdater for AscendExSpotBookUpdater {
    type OrderBook = OrderBook;
    type UpdateEvent = AscendExBookUpdate;

    async fn init(instrument: &Instrument) -> Result<InstrumentOrderBook<Self>, SocketError> {
        // let (snapshot, ticker_infos) = try_join!(
        //     AscendExSpotPublicData::get_book_snapshot(instrument.clone()),
        //     AscendExSpotPublicData::get_ticker_info(instrument.clone()),
        // )?;

        let ticker_infos = AscendExSpotPublicData::get_ticker_info(instrument.clone()).await?;
        std::thread::sleep(std::time::Duration::from_millis(500));

        let snapshot = AscendExSpotPublicData::get_book_snapshot(instrument.clone()).await?;
        std::thread::sleep(std::time::Duration::from_millis(500));

        let ticker_formatted = AssetFormatted::from((&ExchangeId::AscendExSpot, instrument));

        // Note: we cannot get ticker info for one pair we have to get for every single one
        let tick_size = ticker_infos.data.into_iter().find_map(|ticker_info| {
            if ticker_info.symbol == ticker_formatted.0 {
                Some(ticker_info.tick_size)
            } else {
                None
            }
        });

        match tick_size {
            Some(tick_size) => {
                let mut orderbook_init = OrderBook::new(tick_size);
                orderbook_init.process_lvl2(snapshot.data.data.bids, snapshot.data.data.asks);

                Ok(InstrumentOrderBook {
                    instrument: instrument.clone(),
                    updater: Self::new(0),
                    book: orderbook_init,
                })
            }
            None => Err(SocketError::TickSizeError {
                base: instrument.base.clone(),
                quote: instrument.quote.clone(),
                exchange: ExchangeId::AscendExSpot,
            }),
        }
    }

    fn update(
        &mut self,
        book: &mut Self::OrderBook,
        update: Self::UpdateEvent,
    ) -> Result<Option<EventOrderBook>, SocketError> {
        if self.is_first_update() {
            self.validate_first_update(&update)?;
        } else {
            self.validate_next_update(&update)?;
        }

        book.last_update_time = Utc::now();
        book.process_lvl2(update.data.bids, update.data.asks);

        self.updates_processed += 1;
        self.sequence_number = update.data.seqnum;

        Ok(Some(book.book_snapshot()))
    }
}
