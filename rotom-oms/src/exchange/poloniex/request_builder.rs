use chrono::Utc;

use base64::Engine;
use hmac::Mac;
use reqwest::Request;
use rotom_data::{
    error::SocketError,
    protocols::http::{request_builder::ExchangeRequestBuilder, rest_request::RestRequest},
};

use crate::exchange::HmacSha256;

/*----- */
// Poloniex API Authentication
/*----- */
pub struct PoloniexAuthParams;

impl PoloniexAuthParams {
    pub const SECRET: &'static str = env!("POLONIEX_API_SECRET");
    pub const KEY: &'static str = env!("POLONIEX_API_KEY");

    #[inline]
    pub fn generate_signature(request_str: String) -> String {
        let mut mac = HmacSha256::new_from_slice(PoloniexAuthParams::SECRET.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(request_str.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }
}

/*----- */
// Impl Authenticator for Poloniex
/*----- */
#[derive(Debug)]
pub struct PoloniexRequestBuilder;

#[derive(Debug)]
pub struct PoloniexConfig<'a, Request> {
    pub timestamp: i64,
    pub request: &'a Request,
}

impl<'a, Request> PoloniexConfig<'a, Request>
where
    Request: RestRequest,
{
    pub fn new(request: &'a Request) -> Self {
        Self {
            timestamp: Utc::now().timestamp_millis(),
            request,
        }
    }

    pub fn generate_body(&self) -> Result<String, SocketError> {
        Ok(serde_json::to_string(self.request.body().unwrap())
            .map_err(SocketError::Serialise)?
            .replace(',', ", ")
            .replace(':', ": "))
    }

    pub fn generate_query(&self, request_body: &str) -> String {
        let method = Request::method();
        let encoded_params = format!(
            "requestBody={}&signTimestamp={}",
            request_body, self.timestamp
        );

        [
            method.as_str(),
            self.request.path().as_ref(),
            encoded_params.as_str(),
        ]
        .join("\n")
    }
}

impl ExchangeRequestBuilder for PoloniexRequestBuilder {
    type AuthParams = PoloniexAuthParams;

    #[inline]
    fn generate_signature(request_str: impl Into<String>) -> String {
        let mut mac = HmacSha256::new_from_slice(PoloniexAuthParams::SECRET.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(request_str.into().as_bytes());
        base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }

    #[inline]
    fn build_signed_request<Request>(
        builder: reqwest::RequestBuilder,
        request: Request,
    ) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest,
    {
        let request_config = PoloniexConfig::new(&request);
        let request_body = request_config.generate_body()?;
        let request_query = request_config.generate_query(request_body.as_str());
        let signature = Self::generate_signature(request_query);

        let builder = builder
            .header("Content-Type", "application/json")
            .header("key", PoloniexAuthParams::KEY)
            .header("signature", signature)
            .header("signTimestamp", request_config.timestamp)
            .body(request_body)
            .build()
            .map_err(SocketError::from)?;

        Ok(builder)
    }
}
