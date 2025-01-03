use std::borrow::Cow;

use ::serde::{Deserialize, Serialize};
use rotom_data::protocols::http::rest_request::RestRequest;
use uuid::Uuid;

use crate::{
    exchange::errors::RequestBuildError,
    model::{order::OpenOrder, OrderKind},
};
use rotom_data::shared::de::de_str;

use super::{PoloniexOrderType, PoloniexSide, PoloniexSymbol, PoloniexTimeInForce};

/*----- */
// Poloniex New Order
/*----- */
#[derive(Debug, Serialize)]
pub struct PoloniexNewOrder {
    symbol: String,
    side: String,
    #[serde(rename(serialize = "timeInForce"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    time_in_force: Option<PoloniexTimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<PoloniexOrderType>,
    #[serde(rename(serialize = "accountType"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    account_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<String>,
    #[serde(rename(serialize = "clientOrderId"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    client_order_id: Option<Uuid>,
    #[serde(rename(serialize = "allowBorrow"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_borrow: Option<bool>,
    #[serde(rename(serialize = "stpMode"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    stp_mode: Option<String>,
    #[serde(rename(serialize = "slippageTolerance"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    slippage_tolerance: Option<String>,
}

impl PoloniexNewOrder {
    pub fn new(order_event: &OpenOrder) -> Result<Self, RequestBuildError> {
        // Poloniex has a weird create new order api where, market buy needs a
        // amount field, which is price x quantity aka quote asset value. While other
        // ones like limit buy / sell and market sell requires a quantity field
        // https://api-docs.poloniex.com/spot/api/private/order
        match (
            &order_event.order_kind,
            PoloniexSide::from(&order_event.decision),
        ) {
            (OrderKind::Market, PoloniexSide::BUY) => Self::market_buy(order_event),
            (OrderKind::Market, PoloniexSide::SELL) => Self::market_sell(order_event),
            (OrderKind::Limit, PoloniexSide::BUY) => Self::limit_order_and_market_sell(order_event),
            (OrderKind::Limit, PoloniexSide::SELL) => {
                Self::limit_order_and_market_sell(order_event)
            }
            _ => unimplemented!(), // todo
        }
    }

    pub fn limit_order_and_market_sell(order_event: &OpenOrder) -> Result<Self, RequestBuildError> {
        PoloniexNewOrderBuilder::new()
            .symbol(PoloniexSymbol::from(&order_event.instrument).0)
            .side(
                PoloniexSide::from(&order_event.decision)
                    .as_ref()
                    .to_lowercase(),
            )
            .r#type(PoloniexOrderType::Limit) // TODO: limit_marker
            .quantity(order_event.quantity.to_string())
            .price(order_event.price.to_string())
            .time_in_force(PoloniexTimeInForce::GTC) // TODO
            .build()
    }

    pub fn market_buy(order_event: &OpenOrder) -> Result<Self, RequestBuildError> {
        PoloniexNewOrderBuilder::new()
            .symbol(PoloniexSymbol::from(&order_event.instrument).0)
            .side(
                PoloniexSide::from(&order_event.decision)
                    .as_ref()
                    .to_lowercase(),
            )
            .amount(order_event.notional_amount.to_string())
            .build()
    }

    pub fn market_sell(order_event: &OpenOrder) -> Result<Self, RequestBuildError> {
        PoloniexNewOrderBuilder::new()
            .symbol(PoloniexSymbol::from(&order_event.instrument).0)
            .side(
                PoloniexSide::from(&order_event.decision)
                    .as_ref()
                    .to_lowercase(),
            )
            .quantity(order_event.quantity.to_string())
            .build()
    }
}

impl RestRequest for PoloniexNewOrder {
    type Response = PoloniexNewOrderResponse;
    type QueryParams = ();
    type Body = Self;

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/orders")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::POST
    }

    fn body(&self) -> Option<&Self::Body> {
        Some(self)
    }
}

/*----- */
// Poliniex New Order Builder
/*----- */
#[derive(Debug, Serialize, Default)]
pub struct PoloniexNewOrderBuilder {
    symbol: Option<String>,
    side: Option<String>,
    #[serde(rename(serialize = "timeInForce"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    time_in_force: Option<PoloniexTimeInForce>,
    r#type: Option<PoloniexOrderType>,
    #[serde(rename(serialize = "accountType"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    account_type: Option<String>,
    price: Option<String>,
    quantity: Option<String>,
    amount: Option<String>,
    #[serde(rename(serialize = "clientOrderId"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    client_order_id: Option<Uuid>,
    #[serde(rename(serialize = "allowBorrow"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_borrow: Option<bool>,
    #[serde(rename(serialize = "stpMode"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    stp_mode: Option<String>,
    #[serde(rename(serialize = "slippageTolerance"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    slippage_tolerance: Option<String>,
}

impl PoloniexNewOrderBuilder {
    pub fn new() -> Self {
        PoloniexNewOrderBuilder::default()
    }

    pub fn symbol(self, symbol: String) -> Self {
        Self {
            symbol: Some(symbol),
            ..self
        }
    }

    pub fn side(self, side: String) -> Self {
        Self {
            side: Some(side),
            ..self
        }
    }

    pub fn time_in_force(self, time_in_force: PoloniexTimeInForce) -> Self {
        Self {
            time_in_force: Some(time_in_force),
            ..self
        }
    }

    pub fn r#type(self, r#type: PoloniexOrderType) -> Self {
        Self {
            r#type: Some(r#type),
            ..self
        }
    }

    pub fn account_type(self, account_type: String) -> Self {
        Self {
            account_type: Some(account_type),
            ..self
        }
    }

    pub fn price(self, price: String) -> Self {
        Self {
            price: Some(price),
            ..self
        }
    }

    pub fn quantity(self, quantity: String) -> Self {
        Self {
            quantity: Some(quantity),
            ..self
        }
    }

    pub fn amount(self, amount: String) -> Self {
        Self {
            amount: Some(amount),
            ..self
        }
    }

    pub fn client_order_id(self, client_order_id: Uuid) -> Self {
        Self {
            client_order_id: Some(client_order_id),
            ..self
        }
    }

    pub fn allow_borrow(self, allow_borrow: bool) -> Self {
        Self {
            allow_borrow: Some(allow_borrow),
            ..self
        }
    }

    pub fn stp_mode(self, stp_mode: String) -> Self {
        Self {
            stp_mode: Some(stp_mode),
            ..self
        }
    }

    pub fn slippage_tolerance(self, slippage_tolerance: String) -> Self {
        Self {
            slippage_tolerance: Some(slippage_tolerance),
            ..self
        }
    }

    pub fn build(self) -> Result<PoloniexNewOrder, RequestBuildError> {
        Ok(PoloniexNewOrder {
            symbol: self.symbol.ok_or(RequestBuildError::MandatoryField {
                exchange: "Poloniex",
                request: "new order",
                field: "symbol",
            })?,
            side: self.side.ok_or(RequestBuildError::MandatoryField {
                exchange: "Poloniex",
                request: "new order",
                field: "side",
            })?,
            time_in_force: self.time_in_force,
            r#type: self.r#type,
            account_type: self.account_type,
            price: self.price,
            quantity: self.quantity,
            amount: self.amount,
            client_order_id: self.client_order_id,
            allow_borrow: self.allow_borrow,
            stp_mode: self.stp_mode,
            slippage_tolerance: self.slippage_tolerance,
        })
    }
}

/*----- */
// Poloniex New Order Response
/*----- */
#[derive(Debug, Deserialize)]
pub struct PoloniexNewOrderResponse {
    #[serde(deserialize_with = "de_str")]
    pub id: u64,
    #[serde(alias = "clientOrderId")]
    pub client_order_id: String,
}
