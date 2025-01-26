use async_trait::async_trait;
use futures::try_join;

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
    pub sequence_number: u64,
}

impl AscendExSpotBookUpdater {
    pub fn new(last_update_id: u64) -> Self {
        Self { sequence_number: 0 }
    }
}

#[async_trait]
impl OrderBookUpdater for AscendExSpotBookUpdater {
    type OrderBook = OrderBook;
    type UpdateEvent = AscendExBookUpdate;

    async fn init(instrument: &Instrument) -> Result<InstrumentOrderBook<Self>, SocketError> {
        let (snapshot, ticker_infos) = try_join!(
            AscendExSpotPublicData::get_book_snapshot(instrument.clone()),
            AscendExSpotPublicData::get_ticker_info(instrument.clone()),
        )?;

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
                    updater: Self::new(snapshot.data.data.seqnum),
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
        Ok(Some(book.book_snapshot()))
    }
}
