use chrono::Utc;
use rotom_data::protocols::http::request_builder::Authenticator;
use rotom_data::protocols::http::rest_request::RestRequest;
use serde::Deserialize;
use serde::Serialize;
use serde_urlencoded;
use std::borrow::Cow;

use crate::exchange::binance::request_builder::BinanceAuthParams;
use crate::exchange::errors::RequestBuildError;
use crate::model::order::OrderEvent;
use crate::model::OrderKind;
use rotom_data::shared::de::de_str;

use super::BinanceFill;
use super::BinanceOrderStatus;
use super::{BinanceSide, BinanceSymbol, BinanceTimeInForce};

/*----- */
// Binance New Order Params
/*----- */
// Mandatory field: symbol, side, type, apiKey, signature, timestamp
#[derive(Debug, Serialize)]
pub struct BinanceNewOrder {
    #[serde(rename(serialize = "icebergQty"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iceberg_qty: Option<u64>,
    #[serde(rename(serialize = "newClientOrderId"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    #[serde(rename(serialize = "newOrderRespType"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    pub quantity: Option<f64>,
    #[serde(rename(serialize = "quoteOrderQty"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_order_qty: Option<f64>,
    #[serde(rename(serialize = "recvWindow"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>, // value cannot be > 60_000
    #[serde(rename(serialize = "selfTradePreventionMode"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_mode: Option<String>,
    pub side: BinanceSide,
    pub signature: Option<String>,
    #[serde(rename(serialize = "stopPrice"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(rename(serialize = "strategyId"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_id: Option<u64>,
    #[serde(rename(serialize = "strategyType"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_type: Option<u64>,
    pub symbol: String,
    #[serde(rename(serialize = "timeInForce"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<BinanceTimeInForce>,
    pub timestamp: i64,
    pub r#type: String,
    #[serde(rename(serialize = "trailingDelta"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_delta: Option<u64>,
}

impl BinanceNewOrder {
    pub fn new(order_event: &OrderEvent) -> Result<Self, RequestBuildError> {
        match &order_event.order_kind {
            OrderKind::Limit => Self::limit_order(order_event),
            OrderKind::Market => Self::market_order(order_event),
        }
    }

    fn limit_order(order_event: &OrderEvent) -> Result<Self, RequestBuildError> {
        BinanceNewOrderParamsBuilder::default()
            .price(order_event.market_meta.close)
            .quantity(order_event.quantity)
            .side(BinanceSide::from(order_event.decision))
            .symbol(BinanceSymbol::from(&order_event.instrument).0)
            .time_in_force(BinanceTimeInForce::GTC) // todo
            .r#type(order_event.order_kind.as_ref().to_uppercase())
            .sign()
            .build()
    }

    fn market_order(order_event: &OrderEvent) -> Result<Self, RequestBuildError> {
        BinanceNewOrderParamsBuilder::default()
            .quantity(order_event.quantity)
            .side(BinanceSide::from(order_event.decision))
            .symbol(BinanceSymbol::from(&order_event.instrument).0)
            .r#type(order_event.order_kind.as_ref().to_uppercase())
            .sign()
            .build()
    }
}

impl RestRequest for BinanceNewOrder {
    type Response = BinanceNewOrderResponses;
    type QueryParams = Self;
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/order")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::POST
    }

    fn query_params(&self) -> Option<&Self> {
        Some(self)
    }
}

/*----- */
// Binance New Order Param Builder
/*----- */
#[derive(Debug, Serialize)]
pub struct BinanceNewOrderParamsBuilder {
    #[serde(rename(serialize = "icebergQty"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iceberg_qty: Option<u64>,
    #[serde(rename(serialize = "newClientOrderId"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    #[serde(rename(serialize = "newOrderRespType"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    pub quantity: Option<f64>,
    #[serde(rename(serialize = "quoteOrderQty"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_order_qty: Option<f64>,
    #[serde(rename(serialize = "recvWindow"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>, // value cannot be > 60_000
    #[serde(rename(serialize = "selfTradePreventionMode"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_mode: Option<String>,
    pub side: Option<BinanceSide>,
    pub signature: Option<String>,
    #[serde(rename(serialize = "stopPrice"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(rename(serialize = "strategyId"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_id: Option<u64>,
    #[serde(rename(serialize = "strategyType"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_type: Option<u64>,
    pub symbol: Option<String>,
    #[serde(rename(serialize = "timeInForce"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<BinanceTimeInForce>,
    pub timestamp: i64,
    pub r#type: Option<String>,
    #[serde(rename(serialize = "trailingDelta"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_delta: Option<u64>,
}

impl Default for BinanceNewOrderParamsBuilder {
    fn default() -> Self {
        Self {
            iceberg_qty: None,
            new_client_order_id: None,
            new_order_resp_type: None,
            price: None,
            quantity: None,
            quote_order_qty: None,
            recv_window: None,
            self_trade_prevention_mode: None,
            side: None,
            signature: None,
            stop_price: None,
            strategy_id: None,
            strategy_type: None,
            symbol: None,
            time_in_force: None,
            timestamp: Utc::now().timestamp_millis(),
            r#type: None,
            trailing_delta: None,
        }
    }
}

impl BinanceNewOrderParamsBuilder {
    pub fn iceberg_qty(self, iceberg_qty: u64) -> Self {
        Self {
            iceberg_qty: Some(iceberg_qty),
            ..self
        }
    }

    pub fn new_client_order_id(self, new_client_order_id: String) -> Self {
        Self {
            new_client_order_id: Some(new_client_order_id),
            ..self
        }
    }

    pub fn new_order_resp_type(self, new_order_resp_type: String) -> Self {
        Self {
            new_order_resp_type: Some(new_order_resp_type),
            ..self
        }
    }

    pub fn price(self, price: f64) -> Self {
        Self {
            price: Some(price),
            ..self
        }
    }

    pub fn quantity(self, quantity: f64) -> Self {
        Self {
            quantity: Some(quantity),
            ..self
        }
    }

    pub fn quote_order_qty(self, quote_order_qty: f64) -> Self {
        Self {
            quote_order_qty: Some(quote_order_qty),
            ..self
        }
    }

    pub fn recv_window(self, recv_window: u64) -> Self {
        Self {
            recv_window: Some(recv_window),
            ..self
        }
    }

    pub fn self_trade_prevention_mode(self, self_trade_prevention_mode: String) -> Self {
        Self {
            self_trade_prevention_mode: Some(self_trade_prevention_mode),
            ..self
        }
    }

    pub fn side(self, side: BinanceSide) -> Self {
        Self {
            side: Some(side),
            ..self
        }
    }

    pub fn sign(self) -> Self {
        let signature = BinanceAuthParams::generate_signature(
            serde_urlencoded::to_string(&self).unwrap_or_default(),
        );

        Self {
            signature: Some(signature),
            ..self
        }
    }

    pub fn stop_price(self, stop_price: f64) -> Self {
        Self {
            stop_price: Some(stop_price),
            ..self
        }
    }

    pub fn strategy_id(self, strategy_id: u64) -> Self {
        Self {
            strategy_id: Some(strategy_id),
            ..self
        }
    }

    pub fn strategy_type(self, strategy_type: u64) -> Self {
        Self {
            strategy_type: Some(strategy_type),
            ..self
        }
    }

    pub fn symbol(self, symbol: String) -> Self {
        Self {
            symbol: Some(symbol),
            ..self
        }
    }

    pub fn time_in_force(self, time_in_force: BinanceTimeInForce) -> Self {
        Self {
            time_in_force: Some(time_in_force),
            ..self
        }
    }

    pub fn r#type(self, r#type: String) -> Self {
        Self {
            r#type: Some(r#type),
            ..self
        }
    }

    pub fn trailing_delta(self, trailing_delta: u64) -> Self {
        Self {
            trailing_delta: Some(trailing_delta),
            ..self
        }
    }

    pub fn build(self) -> Result<BinanceNewOrder, RequestBuildError> {
        Ok(BinanceNewOrder {
            iceberg_qty: self.iceberg_qty,
            new_client_order_id: self.new_client_order_id,
            new_order_resp_type: self.new_order_resp_type,
            price: self.price,
            quantity: self.quantity,
            quote_order_qty: self.quote_order_qty,
            recv_window: self.recv_window,
            self_trade_prevention_mode: self.self_trade_prevention_mode,
            side: self.side.ok_or(RequestBuildError::MandatoryField {
                exchange: "Binance",
                request: "new order",
                field: "side",
            })?,
            signature: self.signature,
            stop_price: self.stop_price,
            strategy_id: self.strategy_id,
            strategy_type: self.strategy_type,
            symbol: self.symbol.ok_or(RequestBuildError::MandatoryField {
                exchange: "Binance",
                request: "new order",
                field: "symbol",
            })?,
            time_in_force: self.time_in_force,
            timestamp: self.timestamp,
            r#type: self.r#type.ok_or(RequestBuildError::MandatoryField {
                exchange: "Binance",
                request: "new order",
                field: "type",
            })?,
            trailing_delta: self.trailing_delta,
        })
    }
}

/*----- */
// Binance New Order Response
/*----- */
// Expected response after submitting new order trade for Limit & Market order types
// https://developers.binance.com/docs/binance-spot-api-docs/web-socket-api#place-new-order-trade
#[derive(Debug, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum BinanceNewOrderResponses {
    Full(BinanceNewOrderResponseFull),
    Result(BinanceNewOrderResponseResult),
    Ack(BinanceNewOrderResponseAck),
}

#[derive(Debug, Deserialize)]
pub struct BinanceNewOrderResponseResult {
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

#[derive(Debug, Deserialize)]
pub struct BinanceNewOrderResponseFull {
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

#[derive(Debug, Deserialize)]
pub struct BinanceNewOrderResponseAck {
    pub symbol: String,
    #[serde(alias = "orderId")]
    pub order_id: u64,
    #[serde(alias = "orderListId")]
    pub order_list_id: i64,
    #[serde(alias = "clientOrderId")]
    pub client_order_id: String,
    #[serde(alias = "transactTime")]
    pub transact_time: u64,
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
        }"#;

        let response_de = serde_json::from_str::<BinanceNewOrderResponses>(response);
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
        }"#;

        let response_de = serde_json::from_str::<BinanceNewOrderResponses>(response);
        let mut _result = false;

        match response_de {
            Ok(_) => _result = true,
            Err(_) => _result = false,
        }

        assert!(_result)
    }
}
