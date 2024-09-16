use base64::Engine;
use hmac::Mac;

use crate::exchange::HmacSha256;

/*----- */
// Poloniex API Authentication
/*----- */
pub struct PoloniexAuthParams;

impl PoloniexAuthParams {
    pub const SECRET: &'static str = env!("POLONIEX_API_SECRET");
    pub const KEY: &'static str = env!("POLONIEX_API_KEY");
}

#[inline]
pub fn generate_signature(request_str: String) -> String {
    let mut mac = HmacSha256::new_from_slice(PoloniexAuthParams::SECRET.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(request_str.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}
