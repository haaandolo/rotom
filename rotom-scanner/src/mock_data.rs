/*----- */
// Test utils
/*----- */
pub mod test_utils {
    use std::cmp::Ordering;

    use chrono::Utc;
    use rotom_data::{
        assets::level::Level,
        model::{
            event_book::EventOrderBook,
            event_trade::EventTrade,
            market_event::{DataKind, MarketEvent},
            network_info::NetworkSpecs,
        },
        shared::subscription_models::{ExchangeId, Instrument},
    };
    use tokio::sync::mpsc;

    use crate::{scanner::{SpotArbScanner, SpreadChange}, server::server_channels::make_http_channels};

    fn sort_by_price_ascending(levels: &mut [Level]) {
        levels.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(Ordering::Equal));
    }

    fn sort_by_price_descending(levels: &mut [Level]) {
        levels.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(Ordering::Equal));
    }

    pub fn bids() -> Vec<Level> {
        vec![
            Level::new(12.5, 2.5),
            Level::new(11.23, 5.25),
            Level::new(10.29, 10.11),
            Level::new(9.99, 22.44),
            Level::new(8.47, 39.89),
        ]
    }

    pub fn asks() -> Vec<Level> {
        let mut asks = vec![
            Level::new(13.75, 3.23),
            Level::new(13.99, 6.81),
            Level::new(17.19, 11.01),
            Level::new(20.06, 19.46),
            Level::new(23.87, 20.82),
        ];
        sort_by_price_ascending(&mut asks);
        asks
    }

    pub fn bids_random() -> Vec<Level> {
        let mut bids = vec![
            Level::new_random(),
            Level::new_random(),
            Level::new_random(),
            Level::new_random(),
            Level::new_random(),
        ];
        sort_by_price_descending(&mut bids);
        bids
    }

    pub fn asks_random() -> Vec<Level> {
        vec![
            Level::new_random(),
            Level::new_random(),
            Level::new_random(),
            Level::new_random(),
            Level::new_random(),
        ]
    }

    pub fn market_event_orderbook(
        exchange: ExchangeId,
        instrument: Instrument,
    ) -> MarketEvent<DataKind> {
        MarketEvent::<DataKind> {
            exchange_time: Utc::now(),
            received_time: Utc::now(),
            exchange,
            instrument,
            event_data: DataKind::OrderBook(EventOrderBook::new(Utc::now(), bids(), asks())),
        }
    }

    pub fn market_event_orderbook2(
        exchange: ExchangeId,
        instrument: Instrument,
        bids: Vec<Level>,
        asks: Vec<Level>,
    ) -> MarketEvent<DataKind> {
        MarketEvent::<DataKind> {
            exchange_time: Utc::now(),
            received_time: Utc::now(),
            exchange,
            instrument,
            event_data: DataKind::OrderBook(EventOrderBook::new(Utc::now(), bids, asks)),
        }
    }

    pub fn market_event_trade(
        exchange: ExchangeId,
        instrument: Instrument,
    ) -> MarketEvent<DataKind> {
        MarketEvent::<DataKind> {
            exchange_time: Utc::now(),
            received_time: Utc::now(),
            exchange,
            instrument,
            event_data: DataKind::Trade(EventTrade {
                trade: Level::new(13.0, 12.08),
                is_buy: true,
            }),
        }
    }

    pub fn spread_change(
        exchange: ExchangeId,
        instrument: Instrument,
        bid: Option<Level>,
        ask: Option<Level>,
    ) -> SpreadChange {
        SpreadChange {
            exchange,
            instrument,
            bid,
            ask,
        }
    }

    pub fn spot_arb_scanner() -> SpotArbScanner {
        let (_, network_stream_rx) = mpsc::unbounded_channel::<NetworkSpecs>();
        let (_, market_data_stream_rx) = mpsc::unbounded_channel::<MarketEvent<DataKind>>();
        let (scanner_channel, _) = make_http_channels();

        SpotArbScanner::new(network_stream_rx, market_data_stream_rx, scanner_channel)
    }
}
