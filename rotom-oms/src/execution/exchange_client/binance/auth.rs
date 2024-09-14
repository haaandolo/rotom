use hmac::Mac;

use crate::execution::exchange_client::{Authenticator, HmacSha256, ParamString};

/*----- */
// Binance API Authentication
/*----- */
pub struct BinanceAuthParams;

impl BinanceAuthParams {
    pub const SECRET: &'static str = env!("BINANCE_API_SECRET");
    pub const KEY: &'static str = env!("BINANCE_API_KEY");
}

pub struct BinanceAuthenticator;

impl Authenticator for BinanceAuthenticator {
    type AuthParams = BinanceAuthParams;

    #[inline]
    fn generate_signature(request_str: ParamString) -> String {
        let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
            .expect("Could not generate HMAC for Binance");
        mac.update(request_str.0.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}

#[inline]
pub fn generate_signature(request_str: String) -> String {
    let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
        .expect("Could not generate HMAC for Binance");
    mac.update(request_str.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
