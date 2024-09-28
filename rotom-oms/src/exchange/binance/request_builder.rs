use crate::exchange::HmacSha256;
use hmac::Mac;
use rotom_data::{error::SocketError, protocols::http::request_builder::ExchangeRequestBuilder};

/*----- */
// Binance API Authentication
/*----- */
pub struct BinanceAuthParams;

impl  BinanceAuthParams {
    pub const SECRET: &'static str = env!("BINANCE_API_SECRET");
    pub const KEY: &'static str = env!("BINANCE_API_KEY");

    #[inline]
    pub fn generate_signature(request_str: String) -> String {
        let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
            .expect("Could not generate HMAC for Binance");
        mac.update(request_str.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}

/*----- */
// Impl Authenticator for Binance
/*----- */
#[derive(Debug)]
pub struct BinanceRequestBuilder;

impl ExchangeRequestBuilder for BinanceRequestBuilder {
    type AuthParams = BinanceAuthParams;

    #[inline]
    fn generate_signature(request_str: impl Into<String>) -> String {
        let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
            .expect("Could not generate HMAC for Binance");
        mac.update(request_str.into().as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    #[inline]
    fn build_signed_request<Request>(
        builder: reqwest::RequestBuilder,
        _: Request,
    ) -> Result<reqwest::Request, SocketError> {
        Ok(builder
            .header("X-MBX-APIKEY", BinanceAuthParams::KEY)
            .build()?)
    }
}
