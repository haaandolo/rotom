use serde::Deserialize;

use crate::model::{
    balance::{AssetBalance, Balance, BalanceDelta},
    Side,
};

use super::{BinanceOrderStatus, BinanceTimeInForce};
use rotom_data::shared::{de::de_str, subscription_models::ExchangeId};

/*----- */
// Binance User Data - Order
/*----- */
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct BinanceAccountDataOrder {
    pub e: String,             // Event type
    pub E: u64,                // Event time
    pub s: String,             // Symbol
    pub c: String,             // Client order ID
    pub S: Side,               // Side
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
    pub v: Option<i8>, // Prevented Match Id; This is only visible if the order expired due to STP
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

// impl From<BinanceAccountDataOrder> for Order<Open> {
//     fn from(order: BinanceAccountDataOrder) -> Self {
//         Order {
//             exchange: ExchangeId::BinanceSpot,
//             instrument: order.s,
//             client_order_id: ClientOrderId(order.c),
//             side: order.S,
//             state: Open {
//                 id: order.i,
//                 price: order.p,
//                 quantity: 0.0,
//                 filled_quantity: order.Q,
//             },
//         }
//     }
// }

/*----- */
// Binance User Data - Account Update
/*----- */
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct BinanceAccountDataBalance {
    pub e: String,                            // event type
    pub E: u64,                               // event time
    pub u: u64,                               // time of last account update
    pub B: Vec<BinanceAccountDataBalanceVec>, // balance Array
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct BinanceAccountDataBalanceVec {
    pub a: String, // asset
    #[serde(deserialize_with = "de_str")]
    pub f: f64, // free
    #[serde(deserialize_with = "de_str")]
    pub l: f64, // locked
}

impl From<BinanceAccountDataBalance> for Vec<AssetBalance> {
    fn from(account_balances: BinanceAccountDataBalance) -> Self {
        account_balances
            .B
            .into_iter()
            .map(|balance| AssetBalance {
                asset: balance.a,
                exchange: ExchangeId::BinanceSpot,
                balance: Balance {
                    total: balance.f,
                    available: 0.0,
                },
            })
            .collect()
    }
}

/*----- */
// Binance User Data - Balance
/*----- */
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct BinanceAccountDataDelta {
    pub e: String, // event type
    pub E: u64,    // event time
    pub a: String, // asset
    #[serde(deserialize_with = "de_str")]
    pub d: f64, // balance delta
    pub T: u64,    // clear time
}

impl From<BinanceAccountDataDelta> for BalanceDelta {
    fn from(delta: BinanceAccountDataDelta) -> Self {
        BalanceDelta {
            asset: delta.e,
            exchange: ExchangeId::BinanceSpot,
            total: delta.d,
            available: 0.0,
        }
    }
}

/*----- */
// Binance User Data - Order List
/*----- */
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct BinanceAccountDataList {
    pub e: String, // event type
    pub E: u64,    // event time
    pub s: String, // symbol
    pub g: u64,    // order list id
    pub c: String, // contingency type
    pub l: String, // list status type
    pub L: String, // list order status
    pub r: String, // list reject reason
    pub C: String, // list client order id
    pub T: u64,    // transaction time
    pub O: Vec<BinanceUserDataListData>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceUserDataListData {
    pub s: String, // symbol
    pub i: u64,    // order id
    pub c: String, // client order id
}

/*----- */
// Binance User Data - Balance
/*----- */
#[derive(Debug, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
pub enum BinanceAccountEvents {
    Order(BinanceAccountDataOrder),
    BalanceDelta(BinanceAccountDataDelta),
    Balance(BinanceAccountDataBalance),
    List(BinanceAccountDataList),
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

        let response_de = serde_json::from_str::<BinanceAccountEvents>(response);
        let mut _result = false;

        match response_de {
            Ok(_) => _result = true,
            Err(_) => _result = false,
        }

        assert!(_result)
    }
}
