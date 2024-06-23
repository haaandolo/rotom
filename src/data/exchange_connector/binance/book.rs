use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    data::{
        shared::{
            de::{de_str, de_u64_epoch_ms_as_datetime_utc},
            orderbook::{event::Event, level::Level},
        },
        transformer::Transformer,
    },
    error::SocketError,
};
/*----- */
// Models
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceSnapshot {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceBookUpdate {
    #[serde(alias = "E")]
    pub time: u64,
    #[serde(alias = "U")]
    pub first_update_id: u64,
    #[serde(alias = "u")]
    pub last_update_id: u64,
    #[serde(alias = "b")]
    pub bids: Vec<Level>,
    #[serde(alias = "a")]
    pub asks: Vec<Level>,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceTradeUpdate {
    // #[serde(alias = "T", deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    #[serde(alias = "T")]
    pub time: u64,
    #[serde(alias = "t")]
    pub id: u64,
    #[serde(alias = "p", deserialize_with = "de_str")]
    pub price: f64,
    #[serde(alias = "q", deserialize_with = "de_str")]
    pub amount: f64,
    #[serde(alias = "m")]
    pub side: bool,
}

// pub struct Event {
//     pub timestamp: u64,
//     pub seq: u64,
//     pub is_trade: bool,
//     pub is_buy: bool,
//     pub price: f64,
//     pub size: f64,
// }

// Delete later
#[derive(Debug, Deserialize)]
pub struct Response {
    pub result: Option<Vec<String>>,
    pub id: u32,
}

// Delete later
#[derive(Debug, Deserialize)]
pub struct ExpectedResponse {
    pub event: String,
    pub channel: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum BinanceMessage {
    Trade(BinanceTradeUpdate),
    Book(BinanceBookUpdate),
    Snapshot(BinanceSnapshot),
    SubResponse(Response),
    Subscription(ExpectedResponse),
}

/*----- */
// Implement transformer for Binance spot
/*----- */
impl Transformer for BinanceMessage {
    type Error = SocketError;
    type Input = Self;
    type Output = Event;
    type OutputIter = Vec<Result<Self::Output, Self::Error>>;

    fn transform(&mut self, input: Self::Input) -> Self::OutputIter {
        match input {
            BinanceMessage::Trade(trade_update) => {
                vec![Ok(Event::new(
                    trade_update.time,
                    trade_update.id,
                    true,
                    trade_update.side,
                    trade_update.price,
                    trade_update.amount,
                ))]
            }
            BinanceMessage::Book(book_update) => {
                //    let events = Vec::new();
                // bids
                book_update
                    .bids
                    .into_iter()
                    .map(|level| {
                        Ok(Event::new(
                            book_update.time,
                            book_update.first_update_id,
                            false,
                            true,
                            level.price,
                            level.size,
                        ))
                    })
                    .collect::<Vec<Result<Event, SocketError>>>()
            }
            _ => {
                vec![Ok(Event::new(12, 12, false, false, 12.0, 12.0))]
            }
        }
    }
}

/*----- */
// Examples
/*----- */
// let bin_trade = "{\"e\":\"trade\",\"E\":1718097131139,\"s\":\"BTCUSDT\",\"t\":3631373609,\"p\":\"67547.10000000\",\"q\":\"0.00100000\",\"b\":27777962514,\"a\":27777962896,\"T\":1718097131138,\"m\":true,\"M\":true}";
// let bin_book = "{\"e\":\"depthupdate\",\"E\":1718097006844,\"s\":\"btcusdt\",\"U\":47781538300,\"u\":47781538304,\"b\":[[\"67543.58000000\",\"0.03729000\"],[\"67527.08000000\",\"8.71242000\"],[\"67527.06000000\",\"0.00000000\"]],\"a\":[[\"67567.46000000\",\"9.42091000\"]]}";
// let bin_snap = "{\"lastUpdateId\":3476852730,\"bids\":[[\"0.00914500\",\"2.18100000\"],[\"0.00914400\",\"8.12300000\"],[\"0.00914300\",\"16.05300000\"],[\"0.00914200\",\"18.50400000\"],[\"0.00914100\",\"16.79700000\"],[\"0.00914000\",\"1.27200000\"],[\"0.00913900\",\"3.28200000\"],[\"0.00913800\",\"8.49200000\"],[\"0.00913700\",\"10.06900000\"],[\"0.00913600\",\"8.34500000\"]],\"asks\":[[\"0.00914600\",\"312.94200000\"],[\"0.00914700\",\"7.30100000\"],[\"0.00914800\",\"3.14700000\"],[\"0.00914900\",\"19.51300000\"],[\"0.00915000\",\"671.65800000\"],[\"0.00915100\",\"17.09400000\"],[\"0.00915200\",\"12.57100000\"],[\"0.00915300\",\"998.18200000\"],[\"0.00915400\",\"4.03800000\"],[\"0.00915500\",\"338.23200000\"]]}";
