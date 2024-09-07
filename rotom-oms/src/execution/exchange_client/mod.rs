pub mod binance;
pub mod poloniex;

use hmac::Hmac;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::portfolio::OrderEvent;

/*----- */
// Convenient types
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct ParamString(pub String);

type HmacSha256 = Hmac<Sha256>;

/*----- */
// Authenticator
/*----- */
pub trait Authenticator {
    type AuthParams;

    fn generate_signature(param_str: ParamString) -> String;
}

/*----- */
// Trait to convert OrderEvent to Exchange Specific
/*----- */
pub trait OrderEventConverter {
    type OrderKind;
    type ExchangeAuthParams;

    fn get_request_method() -> &'static str;

    fn convert_order_event(order_event: &OrderEvent) -> Self::OrderKind;

    fn get_query_param(&self) -> ParamString
    where
        Self: Serialize;
}
