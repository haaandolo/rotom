pub mod binance;
pub mod poloniex;

use hmac::Hmac;
use sha2::Sha256;

/*----- */
// Convenient types
/*----- */
type HmacSha256 = Hmac<Sha256>;