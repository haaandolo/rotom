use serde::Deserialize;

use super::{BinanceOrderStatus, BinanceSide, BinanceTimeInForce};
use rotom_data::shared::de::de_str;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct BinanceUserData {
    pub e: String,             // Event type
    pub E: u64,                // Event time
    pub s: String,             // Symbol
    pub c: String,             // Client order ID
    pub S: BinanceSide,        // Side
    pub o: String,             // Order type
    pub f: BinanceTimeInForce, // Time in force
    #[serde(deserialize_with = "de_str")]
    pub q: f64, // Order quantity
    #[serde(deserialize_with = "de_str")]
    pub p: f64, // Order price
    #[serde(deserialize_with = "de_str")]
    pub P: f64, // Stop price
    #[serde(deserialize_with = "de_str")]
    pub F: f64, // Iceberg quantity
    pub g: i8,                 // OrderListId
    pub C: String, // Original client order ID; This is the ID of the order being canceled
    pub x: BinanceOrderStatus, // Current execution type
    pub X: BinanceOrderStatus, // Current order status
    pub r: String, // Order reject reason; will be an error code.
    pub i: u64,    // Order ID
    #[serde(deserialize_with = "de_str")]
    pub l: f64, // Last executed quantity
    #[serde(deserialize_with = "de_str")]
    pub z: f64, // Cumulative filled quantity
    #[serde(deserialize_with = "de_str")]
    pub L: f64, // Last executed price
    #[serde(deserialize_with = "de_str")]
    pub n: f64, // Commission amount
    pub N: Option<String>, // Commission asset
    pub T: u64,    // Transaction time
    pub t: i8,     // Trade ID
    pub v: Option<i8>,     // Prevented Match Id; This is only visible if the order expired due to STP
    pub I: u64,    // Ignore
    pub w: bool,   // Is the order on the book?
    pub m: bool,   // Is this trade the maker side?
    pub M: bool,   // Ignore
    pub O: u64,    // Order creation time
    #[serde(deserialize_with = "de_str")]
    pub Z: f64, // Cumulative quote asset transacted quantity
    #[serde(deserialize_with = "de_str")]
    pub Y: f64, // Last quote asset transacted quantity (i.e. lastPrice * lastQty)
    #[serde(deserialize_with = "de_str")]
    pub Q: f64, // Quote Order Quantity
    pub W: u64,    // Working Time; This is only visible if the order has been placed on the book.
    pub V: String, // SelfTradePreventionMode
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_binance_user_data_response() {
        let response = r#"{
            "e":"executionReport",
            "E":1726401153470,
            "s":"OPUSDT",
            "c":"vDxU1T4TnOFLNvMRkjofsM",
            "S":"BUY",
            "o":"LIMIT",
            "f":"GTC",
            "q":"5.00000000",
            "p":"1.42000000",
            "P":"0.00000000",
            "F":"0.00000000",
            "g":-1,
            "C":"",
            "x":"NEW",
            "X":"NEW",
            "r":"NONE",
            "i":1593430628,
            "l":"0.00000000",
            "z":"0.00000000",
            "L":"0.00000000",
            "n":"0",
            "N":null,
            "T":1726401153469,
            "t":-1,
            "I":3278287247,
            "w":true,
            "m":false,
            "M":false,
            "O":1726401153469,
            "Z":"0.00000000",
            "Y":"0.00000000",
            "Q":"0.00000000",
            "W":1726401153469,
            "V":"EXPIRE_MAKER"
        }"#;

        let response_de = serde_json::from_str::<BinanceUserData>(response);
        let mut _result = false;

        match response_de {
            Ok(_) => _result = true,
            Err(_) => _result = false,
        }

        assert!(_result)
    }
}
