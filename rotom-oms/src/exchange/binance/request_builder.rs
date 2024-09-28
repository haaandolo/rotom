use crate::exchange::HmacSha256;
use hmac::Mac;
use rotom_data::{
    error::SocketError,
    protocols::http::{
        request_builder::{Authenticator, ExchangeRequestBuilder},
        rest_request::RestRequest,
    },
};

/*----- */
// Binance API Authentication
/*----- */
pub struct BinanceAuthParams;

impl Authenticator for BinanceAuthParams {
    const SECRET: &'static str = env!("BINANCE_API_SECRET");
    const KEY: &'static str = env!("BINANCE_API_KEY");

    #[inline]
    fn generate_signature(request_str: impl Into<String>) -> String {
        let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
            .expect("Could not generate HMAC for Binance");
        mac.update(request_str.into().as_bytes());
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
    fn build_signed_request<Request>(
        mut builder: reqwest::RequestBuilder,
        request: Request,
    ) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest,
    {
        // Add optional query params
        if let Some(query_params) = request.query_params() {
            builder = builder.query(query_params);
        }

        Ok(builder
            .header("X-MBX-APIKEY", BinanceAuthParams::KEY)
            .build()?)
    }
}
