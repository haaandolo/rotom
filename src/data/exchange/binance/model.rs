use serde::Deserialize;

use crate::data::{
    model::{
        book::EventOrderBook,
        event::MarketEvent,
        level::Level,
        subs::{ExchangeId, StreamType},
        trade::Trade,
    },
    shared::{
        de::{de_str, de_str_symbol, deserialize_non_empty_vec},
        utils::{current_timestamp_utc, snapshot_symbol_default_value},
    },
};

/*----- */
// Snapshot
/*----- */
#[derive(PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceSnapshot {
    #[serde(default = "snapshot_symbol_default_value")]
    symbol: String,
    #[serde(default = "current_timestamp_utc")]
    pub timestamp: u64,
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    pub bids: Option<Vec<Level>>,
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    pub asks: Option<Vec<Level>>,
}

/*----- */
// Orderbook L2
/*----- */
#[derive(PartialEq, PartialOrd, Debug, Deserialize, Default)]
pub struct BinanceBook {
    #[serde(alias = "s")]
    #[serde(alias = "p", deserialize_with = "de_str_symbol")]
    pub symbol: String,
    #[serde(alias = "E")]
    pub timestamp: u64,
    #[serde(alias = "U")]
    pub first_update_id: u64,
    #[serde(alias = "u")]
    pub last_update_id: u64,
    #[serde(alias = "b")]
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    pub bids: Option<Vec<Level>>,
    #[serde(alias = "a")]
    #[serde(deserialize_with = "deserialize_non_empty_vec")]
    pub asks: Option<Vec<Level>>,
}

impl From<BinanceBook> for MarketEvent<EventOrderBook> {
    fn from(event: BinanceBook) -> Self {
        Self {
            exchange_time: event.timestamp,
            received_time: current_timestamp_utc(),
            seq: event.first_update_id,
            exchange: ExchangeId::BinanceSpot,
            stream_type: StreamType::L2,
            symbol: event.symbol,
            event_data: EventOrderBook::new(event.bids, event.asks),
        }
    }
}

/*----- */
// Trade
/*----- */
#[derive(PartialEq, PartialOrd, Debug, Deserialize, Default)]
pub struct BinanceTrade {
    #[serde(alias = "s", deserialize_with = "de_str_symbol")]
    pub symbol: String,
    #[serde(alias = "T")]
    pub timestamp: u64,
    #[serde(alias = "t")]
    pub id: u64,
    #[serde(alias = "p", deserialize_with = "de_str")]
    pub price: f64,
    #[serde(alias = "q", deserialize_with = "de_str")]
    pub amount: f64,
    #[serde(alias = "m")]
    pub side: bool,
}

impl From<BinanceTrade> for MarketEvent<Trade> {
    fn from(event: BinanceTrade) -> Self {
        Self {
            exchange_time: event.timestamp,
            received_time: current_timestamp_utc(),
            seq: event.id,
            exchange: ExchangeId::BinanceSpot,
            stream_type: StreamType::Trades,
            symbol: event.symbol,
            event_data: Trade::new(Level::new(event.price, event.amount), event.side),
        }
    }
}

/*----- */
// Subscription response
/*----- */
#[derive(Debug, Deserialize, PartialEq)]
pub struct BinanceSubscriptionResponse {
    pub result: Option<String>,
    pub id: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged, rename_all = "snake_case")]
pub enum BinanceMessage {
    Book(BinanceBook),
    Snapshot(BinanceSnapshot),
    Trade(BinanceTrade),
}

// /*----- */
// // Tests
// /*----- */
// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn snapshot_de() {
//         let snapshot =  "{\"lastUpdateId\":3476852730,\"bids\":[[\"0.00914500\",\"2.18100000\"],[\"0.00914400\",\"8.12300000\"]],\"asks\":[[\"0.00914600\",\"312.94200000\"],[\"0.00914700\",\"7.30100000\"]]}";
//         let snapshot_de = serde_json::from_str::<BinanceSnapshot>(snapshot).unwrap();
//         let snapshot_expected = BinanceSnapshot {
//             symbol: "snapshot".to_string(),
//             timestamp: current_timestamp_utc(),
//             last_update_id: 3476852730,
//             bids: Some(vec![
//                 Level::new(0.00914500, 2.18100000),
//                 Level::new(0.00914400, 8.12300000),
//             ]),
//             asks: Some(vec![
//                 Level::new(0.00914600, 312.94200000),
//                 Level::new(0.00914700, 7.30100000),
//             ]),
//         };
//         assert_eq!(snapshot_de, snapshot_expected)
//     }

//     #[test]
//     fn snapshot_to_event() {
//         let current_timestamp = current_timestamp_utc();
//         let snapshot_struct = BinanceSnapshot {
//             symbol: "snapshot".to_string(),
//             timestamp: current_timestamp,
//             last_update_id: 3476852730,
//             bids: Some(vec![
//                 Level::new(0.00914500, 2.18100000),
//                 Level::new(0.00914400, 8.12300000),
//             ]),
//             asks: Some(vec![
//                 Level::new(0.00914600, 312.94200000),
//                 Level::new(0.00914700, 7.30100000),
//             ]),
//         };

//         let event_expected = Event::new(
//             "snapshot".to_string(),
//             current_timestamp,
//             3476852730,
//             Some(vec![
//                 Level::new(0.00914500, 2.18100000),
//                 Level::new(0.00914400, 8.12300000),
//             ]),
//             Some(vec![
//                 Level::new(0.00914600, 312.94200000),
//                 Level::new(0.00914700, 7.30100000),
//             ]),
//             None,
//             None,
//         );

//         let event_from = Event::from(snapshot_struct);

//         assert_eq!(event_from, event_expected);
//     }

//     #[test]
//     fn orderbook_de() {
//         let orderbook = "{\"e\":\"depthupdate\",\"E\":1718097006844,\"s\":\"btcusdt\",\"U\":47781538300,\"u\":47781538304,\"b\":[[\"67543.58000000\",\"0.03729000\"],[\"67527.08000000\",\"8.71242000\"]],\"a\":[]}";
//         let orderbook_de = serde_json::from_str::<BinanceBook>(orderbook).unwrap();
//         let orderbook_expected = BinanceBook {
//             symbol: "btcusdt".to_string(),
//             timestamp: 1718097006844,
//             first_update_id: 47781538300,
//             bids: Some(vec![
//                 Level::new(67543.58000000, 0.03729000),
//                 Level::new(67527.08000000, 8.71242000),
//             ]),
//             asks: None,
//         };

//         println!("{:#?}", orderbook_expected);

//         assert_eq!(orderbook_de, orderbook_expected)
//     }

//     #[test]
//     fn orderbook_to_event() {
//         let orderbook_struct = BinanceBook {
//             symbol: "btcusdt".to_string(),
//             timestamp: 1718097006844,
//             first_update_id: 47781538300,
//             bids: Some(vec![
//                 Level::new(67543.58000000, 0.03729000),
//                 Level::new(67527.08000000, 8.71242000),
//             ]),
//             asks: None,
//         };

//         let event_expected = Event::new(
//             "btcusdt".to_string(),
//             1718097006844,
//             47781538300,
//             Some(vec![
//                 Level::new(67543.58000000, 0.03729000),
//                 Level::new(67527.08000000, 8.71242000),
//             ]),
//             None,
//             None,
//             None,
//         );

//         let event_from = Event::from(orderbook_struct);

//         assert_eq!(event_from, event_expected)
//     }

//     #[test]
//     fn trade_de() {
//         let trade = "{\"e\":\"trade\",\"E\":1718097131139,\"s\":\"BTCUSDT\",\"t\":3631373609,\"p\":\"67547.10000000\",\"q\":\"0.00100000\",\"b\":27777962514,\"a\":27777962896,\"T\":1718097131138,\"m\":true,\"M\":true}";
//         let trade_de = serde_json::from_str::<BinanceTrade>(trade).unwrap();
//         let trade_expected = BinanceTrade {
//             symbol: "btcusdt".to_string(),
//             timestamp: 1718097131138,
//             id: 3631373609,
//             price: 67547.10000000,
//             amount: 0.00100000,
//             side: true,
//         };
//         assert_eq!(trade_de, trade_expected)
//     }

//     #[test]
//     fn trade_to_event() {
//         let trade_struct = BinanceTrade {
//             symbol: "btcusdt".to_string(),
//             timestamp: 1718097131138,
//             id: 3631373609,
//             price: 67547.10000000,
//             amount: 0.00100000,
//             side: true,
//         };

//         let event_expected = Event::new(
//             "btcusdt".to_string(),
//             1718097131138,
//             3631373609,
//             None,
//             None,
//             Some(Level::new(67547.10000000, 0.00100000)),
//             Some(true),
//         );

//         let event_from = Event::from(trade_struct);

//         assert_eq!(event_from, event_expected)
//     }

//     #[test]
//     fn expected_response_de() {
//         let expected_response = "{\"result\":null,\"id\":1}";
//         let expected_response_de =
//             serde_json::from_str::<BinanceSubscriptionResponse>(expected_response).unwrap();
//         let expected_response_struct = BinanceSubscriptionResponse {
//             result: None,
//             id: 1,
//         };
//         assert_eq!(expected_response_de, expected_response_struct)
//     }
// }
