use hmac::{Hmac, Mac};
use sha2::Sha256;

/*----- */
// Binance API Authentication
/*----- */
pub struct BinanceAuthParams;

impl BinanceAuthParams {
    pub const SECRET: &'static str = env!("BINANCE_API_SECRET");
    pub const KEY: &'static str = env!("BINANCE_API_KEY");
}

#[inline]
pub fn binance_sign(request_str: String) -> String {
    let mut mac =  Hmac::<Sha256>::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
        .expect("Could not generate HMAC for Binance");
    mac.update(request_str.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
