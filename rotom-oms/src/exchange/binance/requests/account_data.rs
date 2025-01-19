use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::model::{
    balance::Balance,
    account_response::{
        AccountBalance, AccountBalanceDelta, AccountResponse, OrderResponse, OrderStatus,
    },
    OrderKind, Side,
};

use super::BinanceTimeInForce;
use rotom_data::shared::{
    de::{de_str, de_u64_epoch_ms_as_datetime_utc},
    subscription_models::ExchangeId,
};

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
    pub o: OrderKind,          // Order type
    pub f: BinanceTimeInForce, // Time in force
    #[serde(deserialize_with = "de_str")]
    pub q: f64, // Order quantity
    #[serde(deserialize_with = "de_str")]
    pub p: f64, // Order price
    #[serde(deserialize_with = "de_str")]
    pub P: f64, // Stop price
    #[serde(deserialize_with = "de_str")]
    pub F: f64, // Iceberg quantity
    pub g: i64,                // OrderListId
    pub C: String, // Original client order ID; This is the ID of the order being canceled
    pub x: OrderStatus, // Current execution type
    pub X: OrderStatus, // Current order status
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
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub T: DateTime<Utc>, // Transaction time
    pub t: i64,    // Trade ID
    pub v: Option<i8>, // Prevented Match Id; This is only visible if the order expired due to STP
    pub I: u64,    // Ignore
    pub w: bool,   // Is the order on the book?
    pub m: bool,   // Is this trade the maker side?
    pub M: bool,   // Ignore
    #[serde(deserialize_with = "de_u64_epoch_ms_as_datetime_utc")]
    pub O: DateTime<Utc>, // Order creation time
    #[serde(deserialize_with = "de_str")]
    pub Z: f64, // Cumulative quote asset transacted quantity
    #[serde(deserialize_with = "de_str")]
    pub Y: f64, // Last quote asset transacted quantity (i.e. lastPrice * lastQty)
    #[serde(deserialize_with = "de_str")]
    pub Q: f64, // Quote Order Quantity
    pub W: u64,    // Working Time; This is only visible if the order has been placed on the book.
    pub V: String, // SelfTradePreventionMode
}

impl From<BinanceAccountDataOrder> for OrderResponse {
    fn from(order: BinanceAccountDataOrder) -> Self {
        Self {
            exchange: ExchangeId::BinanceSpot,
            client_order_id: order.c,
            asset: order.s,
            current_executed_price: order.L,
            current_executed_quantity: order.l,
            cumulative_base: order.z,
            cumulative_quote: order.Z,
            status: order.X,
            execution_time: order.T,
            side: order.S,
            fee: order.n,
        }
    }
}

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

impl From<BinanceAccountDataBalance> for Vec<AccountBalance> {
    fn from(account_balances: BinanceAccountDataBalance) -> Self {
        account_balances
            .B
            .into_iter()
            .map(|balance| AccountBalance {
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

impl From<BinanceAccountDataDelta> for AccountBalanceDelta {
    fn from(delta: BinanceAccountDataDelta) -> Self {
        AccountBalanceDelta {
            asset: delta.a,
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

impl From<BinanceAccountEvents> for AccountResponse {
    fn from(account_events: BinanceAccountEvents) -> Self {
        match account_events {
            BinanceAccountEvents::Order(order) => {
                AccountResponse::Order(OrderResponse::from(order))
            }
            BinanceAccountEvents::Balance(balance) => {
                AccountResponse::BalanceVec(Vec::<AccountBalance>::from(balance))
            }
            BinanceAccountEvents::BalanceDelta(balance_delta) => {
                AccountResponse::BalanceDelta(AccountBalanceDelta::from(balance_delta))
            }
            _ => unimplemented!(),
        }
    }
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

/*----- */
// Examples
/*----- */
/*
### Limit buy partial fill ###
"{
    \"e\":\"executionReport\",
    \"E\":1733290605972,
    \"s\":\"OPUSDT\",
    \"c\":\"web_51c5f50e3b6f476e859d38f2494e7807\",
    \"S\":\"BUY\",
    \"o\":\"LIMIT\",
    \"f\":\"GTC\",
    \"q\":\"5.00000000\",
    \"p\":\"2.58000000\",
    \"P\":\"0.00000000\",
    \"F\":\"0.00000000\",
    \"g\":-1,
    \"C\":\"\",
    \"x\":\"TRADE\",
    \"X\":\"PARTIALLY_FILLED\",
    \"r\":\"NONE\",
    \"i\":1897337989,
    \"l\":\"3.87000000\",
    \"z\":\"3.87000000\",
    \"L\":\"2.58000000\",
    \"n\":\"0.00387000\",
    \"N\":\"OP\",
    \"T\":1733290605971,
    \"t\":109831731,
    \"I\":3914487009,
    \"w\":false,
    \"m\":true,
    \"M\":true,
    \"O\":1733290515358,
    \"Z\":\"9.98460000\",
    \"Y\":\"9.98460000\",
    \"Q\":\"0.00000000\",
    \"W\":1733290515358,
    \"V\":\"EXPIRE_MAKER\"
}",

"{
    \"e\":\"executionReport\",
    \"E\":1733290800451,
    \"s\":\"OPUSDT\",
    \"c\":\"web_51c5f50e3b6f476e859d38f2494e7807\",
    \"S\":\"BUY\",
    \"o\":\"LIMIT\",
    \"f\":\"GTC\",
    \"q\":\"5.00000000\",
    \"p\":\"2.58000000\",
    \"P\":\"0.00000000\",
    \"F\":\"0.00000000\",
    \"g\":-1,
    \"C\":\"\",
    \"x\":\"TRADE\",
    \"X\":\"FILLED\",
    \"r\":\"NONE\",
    \"i\":1897337989,
    \"l\":\"1.13000000\",
    \"z\":\"5.00000000\",
    \"L\":\"2.58000000\",
    \"n\":\"0.00113000\",
    \"N\":\"OP\",
    \"T\":1733290800450,
    \"t\":109831899,
    \"I\":3914497998,
    \"w\":false,
    \"m\":true,
    \"M\":true,
    \"O\":1733290515358,
    \"Z\":\"12.90000000\",
    \"Y\":\"2.91540000\",
    \"Q\":\"0.00000000\",
    \"W\":1733290515358,
    \"V\":\"EXPIRE_MAKER\"
}"

### Limit sell ###
"{\"e\":\"executionReport\",\"E\":1734745890682,\"s\":\"OPUSDT\",\"c\":\"web_5fd0965995964e7d8d7890b500bc1f5d\",\"S\":\"SELL\",\"o\":\"LIMIT\",\"f\":\"GTC\",\"q\":\"4.96000000\",\"p\":\"1.91000000\",\"P\":\"0.00000000\",\"F\":\"0.00000000\",\"g\":-1,\"C\":\"\",\"x\":\"NEW\",\"X\":\"NEW\",\"r\":\"NONE\",\"i\":1985639387,\"l\":\"0.00000000\",\"z\":\"0.00000000\",\"L\":\"0.00000000\",\"n\":\"0\",\"N\":null,\"T\":1734745890681,\"t\":-1,\"I\":4096624429,\"w\":true,\"m\":false,\"M\":false,\"O\":1734745890681,\"Z\":\"0.00000000\",\"Y\":\"0.00000000\",\"Q\":\"0.00000000\",\"W\":1734745890681,\"V\":\"EXPIRE_MAKER\"}",
OrderResponse: OrderResponse {
    exchange: BinanceSpot,
    client_order_id: "web_5fd0965995964e7d8d7890b500bc1f5d",
    asset: "OPUSDT",
    price: 0.0,
    quantity: 0.0,
    status: New,
    execution_time: 2024-12-21T01:51:30.681Z,
    side: Sell,
    fee: 0.0,
    filled_gross: 0.0,
}

"{\"e\":\"executionReport\",\"E\":1734745936793,\"s\":\"OPUSDT\",\"c\":\"web_5fd0965995964e7d8d7890b500bc1f5d\",\"S\":\"SELL\",\"o\":\"LIMIT\",\"f\":\"GTC\",\"q\":\"4.96000000\",\"p\":\"1.91000000\",\"P\":\"0.00000000\",\"F\":\"0.00000000\",\"g\":-1,\"C\":\"\",\"x\":\"TRADE\",\"X\":\"FILLED\",\"r\":\"NONE\",\"i\":1985639387,\"l\":\"4.96000000\",\"z\":\"4.96000000\",\"L\":\"1.91000000\",\"n\":\"0.00947360\",\"N\":\"USDT\",\"T\":1734745936793,\"t\":114330470,\"I\":4096630531,\"w\":false,\"m\":true,\"M\":true,\"O\":1734745890681,\"Z\":\"9.47360000\",\"Y\":\"9.47360000\",\"Q\":\"0.00000000\",\"W\":1734745890681,\"V\":\"EXPIRE_MAKER\"}",
OrderResponse: OrderResponse {
    exchange: BinanceSpot,
    client_order_id: "web_5fd0965995964e7d8d7890b500bc1f5d",
    asset: "OPUSDT",
    price: 1.91,
    quantity: 4.96,
    status: Filled,
    execution_time: 2024-12-21T01:52:16.793Z,
    side: Sell,
    fee: 0.0094736,
    filled_gross: 9.4736,
}

### Market order buy ###
"{\"e\":\"executionReport\",\"E\":1734745597750,\"s\":\"OPUSDT\",\"c\":\"web_ec703d3d0e82439ca696363f4f0c681c\",\"S\":\"BUY\",\"o\":\"MARKET\",\"f\":\"GTC\",\"q\":\"4.99000000\",\"p\":\"0.00000000\",\"P\":\"0.00000000\",\"F\":\"0.00000000\",\"g\":-1,\"C\":\"\",\"x\":\"NEW\",\"X\":\"NEW\",\"r\":\"NONE\",\"i\":1985620466,\"l\":\"0.00000000\",\"z\":\"0.00000000\",\"L\":\"0.00000000\",\"n\":\"0\",\"N\":null,\"T\":1734745597750,\"t\":-1,\"I\":4096586101,\"w\":true,\"m\":false,\"M\":false,\"O\":1734745597750,\"Z\":\"0.00000000\",\"Y\":\"0.00000000\",\"Q\":\"9.51812000\",\"W\":1734745597750,\"V\":\"EXPIRE_MAKER\"}",
"{\"e\":\"executionReport\",\"E\":1734745597750,\"s\":\"OPUSDT\",\"c\":\"web_ec703d3d0e82439ca696363f4f0c681c\",\"S\":\"BUY\",\"o\":\"MARKET\",\"f\":\"GTC\",\"q\":\"4.99000000\",\"p\":\"0.00000000\",\"P\":\"0.00000000\",\"F\":\"0.00000000\",\"g\":-1,\"C\":\"\",\"x\":\"TRADE\",\"X\":\"FILLED\",\"r\":\"NONE\",\"i\":1985620466,\"l\":\"4.99000000\",\"z\":\"4.99000000\",\"L\":\"1.90700000\",\"n\":\"0.00499000\",\"N\":\"OP\",\"T\":1734745597750,\"t\":114329975,\"I\":4096586102,\"w\":false,\"m\":false,\"M\":true,\"O\":1734745597750,\"Z\":\"9.51593000\",\"Y\":\"9.51593000\",\"Q\":\"9.51812000\",\"W\":1734745597750,\"V\":\"EXPIRE_MAKER\"}",

OrderResponse: OrderResponse {
    exchange: BinanceSpot,
    client_order_id: "web_ec703d3d0e82439ca696363f4f0c681c",
    asset: "OPUSDT",
    price: 0.0,
    quantity: 0.0,
    status: New,
    execution_time: 2024-12-21T01:46:37.750Z,
    side: Buy,
    fee: 0.0,
    filled_gross: 0.0,
}
OrderResponse: OrderResponse {
    exchange: BinanceSpot,
    client_order_id: "web_ec703d3d0e82439ca696363f4f0c681c",
    asset: "OPUSDT",
    price: 1.907,
    quantity: 4.99,
    status: Filled,
    execution_time: 2024-12-21T01:46:37.750Z,
    side: Buy,
    fee: 0.00499,
    filled_gross: 9.51593,
}


### Market order sell ###
"{\"e\":\"executionReport\",\"E\":1734745618076,\"s\":\"OPUSDT\",\"c\":\"web_7c4fcae94537497fb41b87a13927fd0f\",\"S\":\"SELL\",\"o\":\"MARKET\",\"f\":\"GTC\",\"q\":\"4.99000000\",\"p\":\"0.00000000\",\"P\":\"0.00000000\",\"F\":\"0.00000000\",\"g\":-1,\"C\":\"\",\"x\":\"NEW\",\"X\":\"NEW\",\"r\":\"NONE\",\"i\":1985621169,\"l\":\"0.00000000\",\"z\":\"0.00000000\",\"L\":\"0.00000000\",\"n\":\"0\",\"N\":null,\"T\":1734745618075,\"t\":-1,\"I\":4096587500,\"w\":true,\"m\":false,\"M\":false,\"O\":1734745618075,\"Z\":\"0.00000000\",\"Y\":\"0.00000000\",\"Q\":\"0.00000000\",\"W\":1734745618075,\"V\":\"EXPIRE_MAKER\"}",
"{\"e\":\"executionReport\",\"E\":1734745618076,\"s\":\"OPUSDT\",\"c\":\"web_7c4fcae94537497fb41b87a13927fd0f\",\"S\":\"SELL\",\"o\":\"MARKET\",\"f\":\"GTC\",\"q\":\"4.99000000\",\"p\":\"0.00000000\",\"P\":\"0.00000000\",\"F\":\"0.00000000\",\"g\":-1,\"C\":\"\",\"x\":\"TRADE\",\"X\":\"FILLED\",\"r\":\"NONE\",\"i\":1985621169,\"l\":\"4.99000000\",\"z\":\"4.99000000\",\"L\":\"1.90700000\",\"n\":\"0.00951593\",\"N\":\"USDT\",\"T\":1734745618075,\"t\":114329985,\"I\":4096587501,\"w\":false,\"m\":false,\"M\":true,\"O\":1734745618075,\"Z\":\"9.51593000\",\"Y\":\"9.51593000\",\"Q\":\"0.00000000\",\"W\":1734745618075,\"V\":\"EXPIRE_MAKER\"}",

OrderResponse: OrderResponse {
    exchange: BinanceSpot,
    client_order_id: "web_7c4fcae94537497fb41b87a13927fd0f",
    asset: "OPUSDT",
    price: 0.0,
    quantity: 0.0,
    status: New,
    execution_time: 2024-12-21T01:46:58.075Z,
    side: Sell,
    fee: 0.0,
    filled_gross: 0.0,
}
OrderResponse: OrderResponse {
    exchange: BinanceSpot,
    client_order_id: "web_7c4fcae94537497fb41b87a13927fd0f",
    asset: "OPUSDT",
    price: 1.907,
    quantity: 4.99,
    status: Filled,
    execution_time: 2024-12-21T01:46:58.075Z,
    side: Sell,
    fee: 0.00951593,
    filled_gross: 9.51593,
}
*/

/*
### Binanace market buy - Raw ###
SpotBalanceId(
    "binancespot_usdt",
): Balance {
    total: 8.15625876,
    available: 0.0,
},
SpotBalanceId(
    "binancespot_op",
): Balance {
    total: 0.00243518,
    available: 0.0,
},

"{
    \"e\":\"executionReport\",
    \"E\":1736150337154,
    \"s\":\"OPUSDT\",
    \"c\":\"web_45ba4d1c84734d178a599be13fd3cce4\",
    \"S\":\"BUY\",
    \"o\":\"MARKET\",
    \"f\":\"GTC\",
    \"q\":\"2.86000000\",
    \"p\":\"0.00000000\",
    \"P\":\"0.00000000\",
    \"F\":\"0.00000000\",
    \"g\":-1,
    \"C\":\"\",
    \"x\":\"NEW\",
    \"X\":\"NEW\",
    \"r\":\"NONE\",
    \"i\":2048717689,
    \"l\":\"0.00000000\",
    \"z\":\"0.00000000\",
    \"L\":\"0.00000000\",
    \"n\":\"0\",
    \"N\":null,
    \"T\":1736150337153,
    \"t\":-1,
    \"I\":4224366292,
    \"w\":true,
    \"m\":false,
    \"M\":false,
    \"O\":1736150337153,
    \"Z\":\"0.00000000\",
    \"Y\":\"0.00000000\",
    \"Q\":\"6.00000000\",
    \"W\":1736150337153,
    \"V\":\"EXPIRE_MAKER\"
}",

"{
    \"e\":\"executionReport\",
    \"E\":1736150337154,
    \"s\":\"OPUSDT\",
    \"c\":\"web_45ba4d1c84734d178a599be13fd3cce4\",
    \"S\":\"BUY\",
    \"o\":\"MARKET\",
    \"f\":\"GTC\",
    \"q\":\"2.86000000\",
    \"p\":\"0.00000000\",
    \"P\":\"0.00000000\",
    \"F\":\"0.00000000\",
    \"g\":-1,
    \"C\":\"\",
    \"x\":\"TRADE\",
    \"X\":\"FILLED\",
    \"r\":\"NONE\",
    \"i\":2048717689,
    \"l\":\"2.86000000\",
    \"z\":\"2.86000000\",
    \"L\":\"2.09200000\",
    \"n\":\"0.00286000\",
    \"N\":\"OP\",
    \"T\":1736150337153,
    \"t\":115755881,
    \"I\":4224366293,
    \"w\":false,
    \"m\":false,
    \"M\":true,
    \"O\":1736150337153,
    \"Z\":\"5.98312000\",
    \"Y\":\"5.98312000\",
    \"Q\":\"6.00000000\",
    \"W\":1736150337153,
    \"V\":\"EXPIRE_MAKER\"
}",

"{
    \"e\":\"outboundAccountPosition\",
    \"E\":1736150337154,k
    \"u\":1736150337153,k
    \"B\":[
        {
            \"a\":\"BNB\",
            \"f\":\"0.00000000\",
            \"l\":\"0.00000000\"
        },
        {
            \"a\":\"USDT\",
            \"f\":\"2.17313876\",
            \"l\":\"0.00000000\"
        },
        {
            \"a\":\"OP\",
            \"f\":\"2.85957518\",
            \"l\":\"0.00000000\"
        }
    ]
}",

usdt_before = 8.15625876
usdt_after = 2.17313876
diff = 5.98312
fee_base = 0.00286000
fee_price = 2.09200000
fee_quote = 0.00598312



*/
