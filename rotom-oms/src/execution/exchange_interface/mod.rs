pub mod binance;
pub mod poloniex;

use hmac::Hmac;
use sha2::Sha256;

/*----- */
// Convenient types
/*----- */
type HmacSha256 = Hmac<Sha256>;

/*----- */
// SignatureGenerator
/*----- */
pub trait ConnectorPrivate {
    type ApiAuthParams;

    fn url() -> &'static str;

    // fn generate_param_str(order: &OrderEvent) -> String;

    fn generate_signature(param_str: String) -> String;

}