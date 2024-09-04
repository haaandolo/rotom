pub mod binance;
pub mod poloniex;

use hmac::Hmac;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::portfolio::OrderEvent;

/*----- */
// Convenient types
/*----- */
type HmacSha256 = Hmac<Sha256>;

/*----- */
// SignatureGenerator
/*----- */
pub trait PrivateConnector {
    type ApiAuthParams;
    type ExchangeSymbol;

    fn url() -> &'static str;

    fn generate_signature(param_str: String) -> String;
}

/*----- */
// Param string
/*----- */
#[derive(Debug, Deserialize, Serialize)]
pub struct ParamString(pub String);

/*----- */
// Signature Generator
/*----- */
pub trait SignatureGenerator {
    type ApiAuthParams;

    fn generate_signature(param_str: String) -> String;
}