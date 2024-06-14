use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Deserialize a `String` as the desired type.
pub fn de_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let data: &str = serde::de::Deserialize::deserialize(deserializer)?;
    data.parse::<T>().map_err(serde::de::Error::custom)
}

pub fn datetime_utc_from_epoch_duration(
    duration: std::time::Duration,
) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::<chrono::Utc>::from(std::time::UNIX_EPOCH + duration)
}

/// Deserialize a `u64` milliseconds value as `DateTime<Utc>`.
pub fn de_u64_epoch_ms_as_datetime_utc<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    serde::de::Deserialize::deserialize(deserializer).map(|epoch_ms| {
        datetime_utc_from_epoch_duration(std::time::Duration::from_millis(epoch_ms))
    })
}

pub fn de_side_from_buyer_is_maker<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer).map(|buyer_is_maker| {
        if buyer_is_maker == "sell" {
            true
        } else {
            false
        }
    })
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct Level {
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub amount: f64,
}

// Binance book update
#[derive(Deserialize, Debug)]
pub struct PoloniexBook {
    pub symbol: String,
    #[serde(alias = "createTime")]
    pub timestamp: u64,
    pub asks: Vec<Level>,
    pub bids: Vec<Level>,
    #[serde(alias = "lastId")]
    pub last_id: u64,
    pub id: u64,
    pub ts: u64,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub channel: String,
    pub data: Vec<PoloniexBook>,
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct PoloniexTradeData {
    pub symbol: String,
    pub amount: String,
    pub quantity: String,
    #[serde(alias = "takerSide", deserialize_with = "de_side_from_buyer_is_maker")]
    pub is_buy: bool,
    #[serde(alias = "createTime")]
    pub timestamp: u64,
    pub price: String,
    pub id: String,
    pub ts: i64,
}

#[derive(Debug, Deserialize)]
pub struct PoloneixTrades {
    pub channel: String,
    data: Vec<PoloniexTradeData>,
}

// Binance Trade
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceTrade {
    #[serde(alias = "T", deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub time: DateTime<Utc>,
    #[serde(alias = "t")]
    pub id: u64,
    #[serde(alias = "p", deserialize_with = "de_str")]
    pub price: f64,
    #[serde(alias = "q", deserialize_with = "de_str")]
    pub amount: f64,
    #[serde(alias = "m")]
    pub side: bool,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize)]
pub struct BinanceBook {
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
pub struct BinanceSnapshot {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

#[tokio::main]
async fn main() {
    // poloniex book update
    let polo_book = "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096579424,\"asks\":[],\"bids\":[[\"67546.83\",\"0.027962\"],[\"67301.78\",\"0\"]],\"lastId\":1051076040,\"id\":1051076041,\"ts\":1718096579435}],\"action\":\"update\"}";
    let polo_snap = "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096578118,\"asks\":[[\"67547.43\",\"0.039788\"],[\"67547.44\",\"0.001416\"],[\"67590.24\",\"0.140000\"],[\"67590.25\",\"1.071006\"],[\"67595.3\",\"0.040284\"],[\"67624.1\",\"0.040284\"],[\"67642.4\",\"0.080727\"],[\"67662.4\",\"0.014779\"],[\"67666.66\",\"0.000217\"],[\"67666.73\",\"0.009021\"],[\"67690.1\",\"0.118535\"],[\"67702.3\",\"0.05205\"],[\"67704.8\",\"0.118535\"],[\"67714.87\",\"0.04905\"],[\"67720.55\",\"0.00024\"],[\"67724.84\",\"0.000035\"],[\"67735\",\"0.000032\"],[\"67738.44\",\"0.00504\"],[\"67738.6\",\"0.234492\"],[\"67746.36\",\"0.000248\"]],\"bids\":[[\"67546.15\",\"0.238432\"],[\"67546.14\",\"0.000602\"],[\"67546.12\",\"0.040284\"],[\"67546.11\",\"0.005187\"],[\"67544.9\",\"0.005016\"],[\"67500\",\"0.044468\"],[\"67490.1\",\"0.040284\"],[\"67462.4\",\"0.080727\"],[\"67440\",\"0.00012\"],[\"67417.7\",\"0.118535\"],[\"67416.53\",\"0.14\"],[\"67400\",\"0.00144\"],[\"67367\",\"0.118535\"],[\"67348.82\",\"0.002970\"],[\"67342.94\",\"0.014849\"],[\"67333.2\",\"0.234492\"],[\"67332.76\",\"0.006665\"],[\"67325\",\"0.4\"],[\"67302\",\"0.030000\"],[\"67301.78\",\"0.000105\"]],\"lastId\":1051076022,\"id\":1051076023,\"ts\":1718096578303}],\"action\":\"snapshot\"}";
    let polo_book: Data = serde_json::from_str(polo_snap).unwrap();
    // println!("{:#?}", polo_book);

    let polo_trade = "{\"channel\":\"trades\",\"data\":[{\"symbol\":\"BTC_USDT\",\"amount\":\"1684.53544514\",\"quantity\":\"0.024914\",\"takerSide\":\"sell\",\"createTime\":1718096866390,\"price\":\"67614.01\",\"id\":\"95714554\",\"ts\":1718096866402}]}";
    let polo_trade: PoloneixTrades = serde_json::from_str(polo_trade).unwrap();
    // println!("{:#?}", polo_trade);

    /*-------------------------------- */
    /*-------------------------------- */

    // binace trade
    let bin_trade = "{\"e\":\"trade\",\"E\":1718097131139,\"s\":\"BTCUSDT\",\"t\":3631373609,\"p\":\"67547.10000000\",\"q\":\"0.00100000\",\"b\":27777962514,\"a\":27777962896,\"T\":1718097131138,\"m\":true,\"M\":true}";
    let bin_trade: BinanceTrade = serde_json::from_str(bin_trade).unwrap();
    // println!("{:#?}", bin_trade);

    let bin_book = "{\"e\":\"depthupdate\",\"e\":1718097006844,\"s\":\"btcusdt\",\"U\":47781538300,\"u\":47781538304,\"b\":[[\"67543.58000000\",\"0.03729000\"],[\"67527.08000000\",\"8.71242000\"],[\"67527.06000000\",\"0.00000000\"]],\"a\":[[\"67567.46000000\",\"9.42091000\"]]}";
    let bin_book: BinanceBook = serde_json::from_str(bin_book).unwrap();
    // println!("{:#?}", bin_book);

    let bin_snap = "{\"lastUpdateId\":3476852730,\"bids\":[[\"0.00914500\",\"2.18100000\"],[\"0.00914400\",\"8.12300000\"],[\"0.00914300\",\"16.05300000\"],[\"0.00914200\",\"18.50400000\"],[\"0.00914100\",\"16.79700000\"],[\"0.00914000\",\"1.27200000\"],[\"0.00913900\",\"3.28200000\"],[\"0.00913800\",\"8.49200000\"],[\"0.00913700\",\"10.06900000\"],[\"0.00913600\",\"8.34500000\"]],\"asks\":[[\"0.00914600\",\"312.94200000\"],[\"0.00914700\",\"7.30100000\"],[\"0.00914800\",\"3.14700000\"],[\"0.00914900\",\"19.51300000\"],[\"0.00915000\",\"671.65800000\"],[\"0.00915100\",\"17.09400000\"],[\"0.00915200\",\"12.57100000\"],[\"0.00915300\",\"998.18200000\"],[\"0.00915400\",\"4.03800000\"],[\"0.00915500\",\"338.23200000\"]]}";
    let bin_snap: BinanceSnapshot = serde_json::from_str(bin_snap).unwrap();
    println!("{:#?}", bin_snap);






     
    
}

/*------------------------------- */
// Poloniex data
/*------------------------------- */
// use arb_bot::data::{subscriber::StreamBuilder, Exchange, StreamType, Sub};
// use serde::{de::{MapAccess, Visitor}, Deserialize};

// #[tokio::main]
// async fn main() {
////////////////////
// // Build Streams
// let mut streams = StreamBuilder::new()
//     .subscribe(vec![
//         // Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
//         // Sub::new(Exchange::BinanceSpot, "arb", "usdt", StreamType::Trades),
//         Sub::new(Exchange::BinanceSpot, "btc", "usdt", StreamType::L2),
//     ])
//     .subscribe(vec![
//         // Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
//         // Sub::new(Exchange::PoloniexSpot, "arb", "usdt", StreamType::Trades),
//         Sub::new(Exchange::PoloniexSpot, "btc", "usdt", StreamType::Trades),
//     ])
//     .init()
//     .await;

// // Read from socket
// if let Some(mut receiver) = streams.remove(&Exchange::BinanceSpot) {
//     while let Some(msg) = receiver.recv().await {
//         // Some(msg);
//         println!("----- Binance -----");
//         println!("{:#?}", msg);
//     }
// }

// // Read from socket
// if let Some(mut receiver) = streams.remove(&Exchange::PoloniexSpot) {
//     while let Some(msg) = receiver.recv().await {
//         // Some(msg);
//         println!("----- Poloniex -----");
//         println!("{:#?}", msg);
//     }
// }
// }

// todo
// - serde json to update book
// - ws auto reconnect
// - expected responses for binance spot and poloniex spot
// - write test for the subscribe fn in stream builder

/*------------------------------- */
// Poloniex data
/*------------------------------- */

// Book
// Text(
//     "{\"channel\":\"book_lv2\",
//       \"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096578118,
//       \"asks\":[[\"67547.43\",\"0.039788\"],[\"67547.44\",\"0.001416\"],[\"67590.24\",\"0.140000\"],[\"67590.25\",\"1.071006\"],[\"67595.3\",\"0.040284\"],[\"67624.1\",\"0.040284\"],[\"67642.4\",\"0.080727\"],[\"67662.4\",\"0.014779\"],[\"67666.66\",\"0.000217\"],[\"67666.73\",\"0.009021\"],[\"67690.1\",\"0.118535\"],[\"67702.3\",\"0.05205\"],[\"67704.8\",\"0.118535\"],[\"67714.87\",\"0.04905\"],[\"67720.55\",\"0.00024\"],[\"67724.84\",\"0.000035\"],[\"67735\",\"0.000032\"],[\"67738.44\",\"0.00504\"],[\"67738.6\",\"0.234492\"],[\"67746.36\",\"0.000248\"]],
//       \"bids\":[[\"67546.15\",\"0.238432\"],[\"67546.14\",\"0.000602\"],[\"67546.12\",\"0.040284\"],[\"67546.11\",\"0.005187\"],[\"67544.9\",\"0.005016\"],[\"67500\",\"0.044468\"],[\"67490.1\",\"0.040284\"],[\"67462.4\",\"0.080727\"],[\"67440\",\"0.00012\"],[\"67417.7\",\"0.118535\"],[\"67416.53\",\"0.14\"],[\"67400\",\"0.00144\"],[\"67367\",\"0.118535\"],[\"67348.82\",\"0.002970\"],[\"67342.94\",\"0.014849\"],[\"67333.2\",\"0.234492\"],[\"67332.76\",\"0.006665\"],[\"67325\",\"0.4\"],[\"67302\",\"0.030000\"],[\"67301.78\",\"0.000105\"]],\"lastId\":1051076022,
//       \"id\":1051076023,
//       \"ts\":1718096578303}],
//       \"action\":\"snapshot\"}",
// )

// "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096578118,\"asks\":[[\"67547.43\",\"0.039788\"],[\"67547.44\",\"0.001416\"],[\"67590.24\",\"0.140000\"],[\"67590.25\",\"1.071006\"],[\"67595.3\",\"0.040284\"],[\"67624.1\",\"0.040284\"],[\"67642.4\",\"0.080727\"],[\"67662.4\",\"0.014779\"],[\"67666.66\",\"0.000217\"],[\"67666.73\",\"0.009021\"],[\"67690.1\",\"0.118535\"],[\"67702.3\",\"0.05205\"],[\"67704.8\",\"0.118535\"],[\"67714.87\",\"0.04905\"],[\"67720.55\",\"0.00024\"],[\"67724.84\",\"0.000035\"],[\"67735\",\"0.000032\"],[\"67738.44\",\"0.00504\"],[\"67738.6\",\"0.234492\"],[\"67746.36\",\"0.000248\"]],\"bids\":[[\"67546.15\",\"0.238432\"],[\"67546.14\",\"0.000602\"],[\"67546.12\",\"0.040284\"],[\"67546.11\",\"0.005187\"],[\"67544.9\",\"0.005016\"],[\"67500\",\"0.044468\"],[\"67490.1\",\"0.040284\"],[\"67462.4\",\"0.080727\"],[\"67440\",\"0.00012\"],[\"67417.7\",\"0.118535\"],[\"67416.53\",\"0.14\"],[\"67400\",\"0.00144\"],[\"67367\",\"0.118535\"],[\"67348.82\",\"0.002970\"],[\"67342.94\",\"0.014849\"],[\"67333.2\",\"0.234492\"],[\"67332.76\",\"0.006665\"],[\"67325\",\"0.4\"],[\"67302\",\"0.030000\"],[\"67301.78\",\"0.000105\"]],\"lastId\":1051076022,\"id\":1051076023,\"ts\":1718096578303}],\"action\":\"snapshot\"}",

// Text(
//     "{\"channel\":\"book_lv2\",
//     \"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096579424,
//     \"asks\":[],
//     \"bids\":[[\"67546.83\",\"0.027962\"],[\"67301.78\",\"0\"]],
//     \"lastId\":1051076040,
//     \"id\":1051076041,
//     \"ts\":1718096579435}],
//     \"action\":\"update\"}",
// )

//     "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096579424,\"asks\":[],\"bids\":[[\"67546.83\",\"0.027962\"],[\"67301.78\",\"0\"]],\"lastId\":1051076040,\"id\":1051076041,\"ts\":1718096579435}],\"action\":\"update\"}",

// Trade
// Text(
//     "{\"channel\":\"trades\",
//     \"data\":[{\"symbol\":\"BTC_USDT\",
//     \"amount\":\"1684.53544514\",
//     \"quantity\":\"0.024914\",
//     \"takerSide\":\"sell\",
//     \"createTime\":1718096866390,
//     \"price\":\"67614.01\",
//     \"id\":\"95714554\",
//     \"ts\":1718096866402}]}",
// )

//     "{\"channel\":\"trades\",\"data\":[{\"symbol\":\"BTC_USDT\",\"amount\":\"1684.53544514\",\"quantity\":\"0.024914\",\"takerSide\":\"sell\",\"createTime\":1718096866390,\"price\":\"67614.01\",\"id\":\"95714554\",\"ts\":1718096866402}]}",

/*------------------------ */
// Binance
/*------------------------ */

// Book
// Text(
//     "{\"e\":\"depthUpdate\",
//     \"E\":1718097006844,
//     \"s\":\"BTCUSDT\",
//     \"U\":47781538300,
//     \"u\":47781538304,
//     \"b\":[[\"67543.58000000\",\"0.03729000\"],[\"67527.08000000\",\"8.71242000\"],[\"67527.06000000\",\"0.00000000\"]],
//     \"a\":[[\"67567.46000000\",\"9.42091000\"]]}",
// )

//     "{\"e\":\"depthUpdate\",\"E\":1718097006844,\"s\":\"BTCUSDT\",\"U\":47781538300,\"u\":47781538304,\"b\":[[\"67543.58000000\",\"0.03729000\"],[\"67527.08000000\",\"8.71242000\"],[\"67527.06000000\",\"0.00000000\"]],\"a\":[[\"67567.46000000\",\"9.42091000\"]]}";

// Trades
// Text(
//     "{\"e\":\"trade\",
//     \"E\":1718097131139,
//     \"s\":\"BTCUSDT\",
//     \"t\":3631373609,
//     \"p\":\"67547.10000000\",
//     \"q\":\"0.00100000\",
//     \"b\":27777962514,
//     \"a\":27777962896,
//     \"T\":1718097131138,
//     \"m\":true,
//     \"M\":true}",
// )

//"{\"e\":\"trade\",\"E\":1718097131139,\"s\":\"BTCUSDT\",\"t\":3631373609,\"p\":\"67547.10000000\",\"q\":\"0.00100000\",\"b\":27777962514,\"a\":27777962896,\"T\":1718097131138,\"m\":true,\"M\":true}"

// Expected response
// Text(
//     "{\"result\":null,\"id\":1}",
// )


// snapshot
// "{\"lastUpdateId\":3476852730,\"bids\":[[\"0.00914500\",\"2.18100000\"],[\"0.00914400\",\"8.12300000\"],[\"0.00914300\",\"16.05300000\"],[\"0.00914200\",\"18.50400000\"],[\"0.00914100\",\"16.79700000\"],[\"0.00914000\",\"1.27200000\"],[\"0.00913900\",\"3.28200000\"],[\"0.00913800\",\"8.49200000\"],[\"0.00913700\",\"10.06900000\"],[\"0.00913600\",\"8.34500000\"]],\"asks\":[[\"0.00914600\",\"312.94200000\"],[\"0.00914700\",\"7.30100000\"],[\"0.00914800\",\"3.14700000\"],[\"0.00914900\",\"19.51300000\"],[\"0.00915000\",\"671.65800000\"],[\"0.00915100\",\"17.09400000\"],[\"0.00915200\",\"12.57100000\"],[\"0.00915300\",\"998.18200000\"],[\"0.00915400\",\"4.03800000\"],[\"0.00915500\",\"338.23200000\"]]}"

// let bin_sc = reqwest::get(" https://api.binance.com/api/v3/depth?symbol=BNBBTC&limit=10")
//     .await
//     .unwrap()
//     .text()
//     .await
//     .unwrap();

// println!("{:#?}", bin_sc);