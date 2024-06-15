use serde::Deserialize;

use crate::data::shared::orderbook::level::Level;

/*---------- */
// Models
/*---------- */
#[derive(Deserialize, Debug)]
pub struct PoloniexBookData {
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
pub struct PoloniexBookDelta {
    pub channel: String,
    pub data: Vec<PoloniexBookData>,
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct PoloniexTradeData {
    pub symbol: String,
    pub amount: String,
    pub quantity: String,
    #[serde(alias = "takerSide", deserialize_with = "de_buyer_is_maker_poloniex")]
    pub is_buy: bool,
    #[serde(alias = "createTime")]
    pub timestamp: u64,
    pub price: String,
    pub id: String,
    pub ts: i64,
}

#[derive(Debug, Deserialize)]
pub struct PoloneixTradeUpdate {
    pub channel: String,
    pub data: Vec<PoloniexTradeData>,
}

/*---------- */
// Exchange specific de
/*---------- */
pub fn de_buyer_is_maker_poloniex<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

/*---------- */
// Examples
/*---------- */
// let polo_book = "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096579424,\"asks\":[],\"bids\":[[\"67546.83\",\"0.027962\"],[\"67301.78\",\"0\"]],\"lastId\":1051076040,\"id\":1051076041,\"ts\":1718096579435}],\"action\":\"update\"}";
// let polo_snap = "{\"channel\":\"book_lv2\",\"data\":[{\"symbol\":\"BTC_USDT\",\"createTime\":1718096578118,\"asks\":[[\"67547.43\",\"0.039788\"],[\"67547.44\",\"0.001416\"],[\"67590.24\",\"0.140000\"],[\"67590.25\",\"1.071006\"],[\"67595.3\",\"0.040284\"],[\"67624.1\",\"0.040284\"],[\"67642.4\",\"0.080727\"],[\"67662.4\",\"0.014779\"],[\"67666.66\",\"0.000217\"],[\"67666.73\",\"0.009021\"],[\"67690.1\",\"0.118535\"],[\"67702.3\",\"0.05205\"],[\"67704.8\",\"0.118535\"],[\"67714.87\",\"0.04905\"],[\"67720.55\",\"0.00024\"],[\"67724.84\",\"0.000035\"],[\"67735\",\"0.000032\"],[\"67738.44\",\"0.00504\"],[\"67738.6\",\"0.234492\"],[\"67746.36\",\"0.000248\"]],\"bids\":[[\"67546.15\",\"0.238432\"],[\"67546.14\",\"0.000602\"],[\"67546.12\",\"0.040284\"],[\"67546.11\",\"0.005187\"],[\"67544.9\",\"0.005016\"],[\"67500\",\"0.044468\"],[\"67490.1\",\"0.040284\"],[\"67462.4\",\"0.080727\"],[\"67440\",\"0.00012\"],[\"67417.7\",\"0.118535\"],[\"67416.53\",\"0.14\"],[\"67400\",\"0.00144\"],[\"67367\",\"0.118535\"],[\"67348.82\",\"0.002970\"],[\"67342.94\",\"0.014849\"],[\"67333.2\",\"0.234492\"],[\"67332.76\",\"0.006665\"],[\"67325\",\"0.4\"],[\"67302\",\"0.030000\"],[\"67301.78\",\"0.000105\"]],\"lastId\":1051076022,\"id\":1051076023,\"ts\":1718096578303}],\"action\":\"snapshot\"}";
// let polo_trade = "{\"channel\":\"trades\",\"data\":[{\"symbol\":\"BTC_USDT\",\"amount\":\"1684.53544514\",\"quantity\":\"0.024914\",\"takerSide\":\"sell\",\"createTime\":1718096866390,\"price\":\"67614.01\",\"id\":\"95714554\",\"ts\":1718096866402}]}";
