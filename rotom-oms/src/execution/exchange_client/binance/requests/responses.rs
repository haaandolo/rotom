use rotom_data::shared::de::de_str;
use serde::Deserialize;

use super::{BinanceSide, BinanceTimeInForce};

/*----- */
// Main Deserialisation Enum for Binance Responses
/*----- */
#[derive(Debug, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum BinanceResponses {
    NewOrderResult(BinanceNewOrderResponseResult),
    NewOrderFull(BinanceNewOrderResponseFull),
    CancelOrder(BinanceCancelOrderResponse),
}

/*----- */
// Component Enum and Structs for Responses
/*----- */
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum BinanceOrderStatus {
    New,
    PendingNew,
    PartiallyFilled,
    Filled,
    Canceled,
    PendingCancel,
    Rejected,
    Expired,
    ExpiredInMatch,
}

#[derive(Debug, Deserialize)]
pub struct RateLimit {
    #[serde(alias = "rateLimitType")]
    pub rate_limit_type: String,
    pub interval: String,
    #[serde(alias = "intervalNum")]
    pub interval_num: u32,
    pub limit: u32,
    pub count: u32,
}

#[derive(Debug, Deserialize)]
pub struct BinanceFill {
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    pub qty: f64,
    #[serde(deserialize_with = "de_str")]
    pub commission: f64,
    #[serde(alias = "commissionAsset")]
    pub commission_asset: String,
    #[serde(alias = "tradeId")]
    pub trade_id: u64,
}

/*----- */
// New Order Response (Result)
/*----- */
// TODO: ACK reponse type
// Expected response after submitting new order trade for Limit & Market order types
// https://developers.binance.com/docs/binance-spot-api-docs/web-socket-api#place-new-order-trade
#[derive(Debug, Deserialize)]
pub struct BinanceNewOrderResponseResult {
    pub id: String,
    pub status: u32,
    pub result: BinanceNewOrderResponseResultData,
    #[serde(alias = "rateLimits")]
    pub rate_limits: Vec<RateLimit>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceNewOrderResponseResultData {
    pub symbol: String,
    #[serde(alias = "orderId")]
    pub order_id: u64,
    #[serde(alias = "orderListId")]
    pub order_list_id: i64,
    #[serde(alias = "clientOrderId")]
    pub client_order_id: String,
    #[serde(alias = "transactTime")]
    pub transact_time: u64,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    #[serde(alias = "origQty")]
    pub orig_qty: f64,
    #[serde(deserialize_with = "de_str")]
    #[serde(alias = "executedQty")]
    pub executed_qty: f64,
    #[serde(deserialize_with = "de_str")]
    #[serde(alias = "cummulativeQuoteQty")]
    pub cummulative_quote_qty: f64,
    pub status: BinanceOrderStatus,
    #[serde(alias = "timeInForce")]
    pub time_in_force: BinanceTimeInForce,
    pub r#type: String,
    pub side: String,
    #[serde(alias = "workingTime")]
    pub working_time: u64,
    pub fills: Vec<BinanceFill>,
    #[serde(alias = "selfTradePreventionMode")]
    pub self_trade_prevention_mode: String,
}

/*----- */
// New Order Response (Full)
/*----- */
#[derive(Debug, Deserialize)]
pub struct BinanceNewOrderResponseFull {
    pub id: String,
    pub status: u32,
    pub result: BinanceNewOrderResponseFullData,
    #[serde(alias = "rateLimits")]
    pub rate_limits: Vec<RateLimit>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceNewOrderResponseFullData {
    pub symbol: String,
    #[serde(alias = "orderId")]
    pub order_id: u64,
    #[serde(alias = "orderListId")]
    pub order_list_id: i64,
    #[serde(alias = "clientOrderId")]
    pub client_order_id: String,
    #[serde(alias = "transactTime")]
    pub transact_time: u64,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str")]
    #[serde(alias = "origQty")]
    pub orig_qty: f64,
    #[serde(deserialize_with = "de_str")]
    #[serde(alias = "executedQty")]
    pub executed_qty: f64,
    #[serde(deserialize_with = "de_str")]
    #[serde(alias = "cummulativeQuoteQty")]
    pub cummulative_quote_qty: f64,
    pub status: BinanceOrderStatus,
    #[serde(alias = "timeInForce")]
    pub time_in_force: BinanceTimeInForce,
    pub r#type: String,
    pub side: String,
    #[serde(alias = "workingTime")]
    pub working_time: u64,
    pub fills: Vec<BinanceFill>,
}

/*----- */
// Cancel order
/*----- */
#[derive(Debug, Deserialize)]
pub struct BinanceCancelOrderResponse {
    pub id: String,
    pub status: u32,
    pub result: BinanceCancelOrderResponseData,
    #[serde(alias = "rateLimits")]
    pub rate_limits: Vec<RateLimit>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceCancelOrderResponseData {
    pub symbol: String,
    #[serde(alias = "origClientOrderId")]
    pub orig_client_order_id: String,
    #[serde(alias = "orderId")]
    pub order_id: u64,
    #[serde(alias = "orderListId")]
    pub order_list_id: i64,
    #[serde(alias = "clientOrderId")]
    pub client_order_id: String,
    #[serde(alias = "transactTime")]
    pub transact_time: u64,
    #[serde(deserialize_with = "de_str")]
    pub price: f64,
    #[serde(deserialize_with = "de_str", alias = "origQty")]
    pub orig_qty: f64,
    #[serde(deserialize_with = "de_str", alias = "executedQty")]
    pub executed_qty: f64,
    #[serde(deserialize_with = "de_str", alias = "cummulativeQuoteQty")]
    pub cummulative_quote_qty: f64,
    pub status: BinanceOrderStatus,
    #[serde(alias = "timeInForce")]
    pub time_in_force: BinanceTimeInForce,
    pub r#type: String, // Change to binance type
    pub side: BinanceSide,
    #[serde(alias = "stopPrice")]
    pub stop_price: Option<String>, // TODO: deserialise into a f64
    #[serde(alias = "trailingDelta")]
    pub trailing_delta: Option<u64>,
    #[serde(alias = "icebergQty")]
    pub iceberg_qty: Option<String>, // TODO: deserialise into a f64
    #[serde(alias = "strategyId")]
    pub strategy_id: Option<u64>,
    #[serde(alias = "strategyType")]
    pub strategy_type: Option<u64>,
    #[serde(alias = "selfTradePreventionMode")]
    pub self_trade_prevention_mode: String,
}

/*----- */
// Tests
/*----- */
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_binance_new_order_response_result() {
        let response = r#"{
            "id": "9c7af916-4126-45ef-b31e-9292ed446842",
            "status": 200,
            "result": {
                "symbol": "OPUSDT",
                "orderId": 1579758355,
                "orderListId": -1,
                "clientOrderId": "JFPdJKG8EfAGLAsTJoYNwQ",
                "transactTime": 1725690996037,
                "price": "0.00000000",
                "origQty": "4.00000000",
                "executedQty": "4.00000000",
                "cummulativeQuoteQty": "5.51600000",
                "status": "FILLED",
                "timeInForce": "GTC",
                "type": "MARKET",
                "side": "BUY",
                "workingTime": 1725690996037,
                "fills": [
                    {
                        "price": "1.37900000",
                        "qty": "4.00000000",
                        "commission": "0.00400000",
                        "commissionAsset": "OP",
                        "tradeId": 89241090
                    }
                ],
                "selfTradePreventionMode": "EXPIRE_MAKER"
            },
            "rateLimits": [
                {
                    "rateLimitType": "ORDERS",
                    "interval": "SECOND",
                    "intervalNum": 10,
                    "limit": 100,
                    "count": 1
                },
                {
                    "rateLimitType": "ORDERS",
                    "interval": "DAY",
                    "intervalNum": 1,
                    "limit": 200000,
                    "count": 1
                },
                {
                    "rateLimitType": "REQUEST_WEIGHT",
                    "interval": "MINUTE",
                    "intervalNum": 1,
                    "limit": 6000,
                    "count": 3
                }
            ]
        }"#;

        let response_de = serde_json::from_str::<BinanceResponses>(response);
        let mut _result = false;

        match response_de {
            Ok(_) => _result = true,
            Err(_) => _result = false,
        }

        assert!(_result)
    }

    #[test]
    fn test_binance_new_order_response_full() {
        let response = r#"{
            "id": "56374a46-3061-486b-a311-99ee972eb648",
            "status": 200,
            "result": {
                "symbol": "BTCUSDT",
                "orderId": 12569099453,
                "orderListId": -1,
                "clientOrderId": "4d96324ff9d44481926157ec08158a40",
                "transactTime": 1660801715793,
                "price": "23416.10000000",
                "origQty": "0.00847000",
                "executedQty": "0.00847000",
                "cummulativeQuoteQty": "198.33521500",
                "status": "FILLED",
                "timeInForce": "GTC",
                "type": "LIMIT",
                "side": "SELL",
                "workingTime": 1660801715793,
                "fills": [
                {
                    "price": "23416.10000000",
                    "qty": "0.00635000",
                    "commission": "0.000000",
                    "commissionAsset": "BNB",
                    "tradeId": 1650422481
                },
                {
                    "price": "23416.50000000",
                    "qty": "0.00212000",
                    "commission": "0.000000",
                    "commissionAsset": "BNB",
                    "tradeId": 1650422482
                }
                ]
            },
            "rateLimits": [
                {
                    "rateLimitType": "ORDERS",
                    "interval": "SECOND",
                    "intervalNum": 10,
                    "limit": 50,
                    "count": 1
                },
                {
                    "rateLimitType": "ORDERS",
                    "interval": "DAY",
                    "intervalNum": 1,
                    "limit": 160000,
                    "count": 1
                },
                {
                    "rateLimitType": "REQUEST_WEIGHT",
                    "interval": "MINUTE",
                    "intervalNum": 1,
                    "limit": 6000,
                    "count": 1
                }
            ]
        }"#;

        let response_de = serde_json::from_str::<BinanceResponses>(response);
        let mut _result = false;

        match response_de {
            Ok(_) => _result = true,
            Err(_) => _result = false,
        }

        assert!(_result)
    }

    #[test]
    fn test_binance_cancel() {
        let response = r#"{
            "id": "5633b6a2-90a9-4192-83e7-925c90b6a2fd",
            "status": 200,
            "result": {
                "symbol": "BTCUSDT",
                "origClientOrderId": "4d96324ff9d44481926157",
                "orderId": 12569099453,
                "orderListId": -1,
                "clientOrderId": "91fe37ce9e69c90d6358c0",
                "transactTime": 1684804350068,
                "price": "23416.10000000",
                "origQty": "0.00847000",
                "executedQty": "0.00001000",
                "cummulativeQuoteQty": "0.23416100",
                "status": "CANCELED",
                "timeInForce": "GTC",
                "type": "LIMIT",
                "side": "SELL",
                "trailingDelta": 0,           
                "icebergQty": "0.00000000",
                "stopPrice": "90.90",
                "strategyId": 37463720,    
                "strategyType": 1000000,     
                "selfTradePreventionMode": "NONE"
            },
            "rateLimits": [
                {
                "rateLimitType": "REQUEST_WEIGHT",
                "interval": "MINUTE",
                "intervalNum": 1,
                "limit": 6000,
                "count": 1
                }
            ]
        }"#;

        let response_de = serde_json::from_str::<BinanceResponses>(response);
        let mut _result = false;

        match response_de {
            Ok(_) => _result = true,
            Err(_) => _result = false,
        }

        assert!(_result)
    }
}

/*
Some(
    Err(
        Deserialise {
            error: Error("data did not match any variant of untagged enum BinanceResponses", line: 0, column: 0),
            payload: "{\"id\":\"6d4fde03-91d7-4943-83bc-9b0ebe7c85cc\",\"status\":400,\"error\":{\"code\":-1021,\"msg\":\"Timestamp for this request is outside of the recvWindow.\"},\"rateLimits\":[{\"rateLimitType\":\"REQUEST_WEIGHT\",\"interval\":\"MINUTE\",\"intervalNum\":1,\"limit\":6000,\"count\":3}]}",
        },
    ),
)
*/