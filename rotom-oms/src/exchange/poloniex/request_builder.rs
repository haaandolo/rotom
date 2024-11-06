use base64::Engine;
use chrono::Utc;
use hmac::Mac;
use std::fmt::Debug;

use rotom_data::{
    error::SocketError,
    protocols::http::{
        request_builder::{Authenticator, ExchangeRequestBuilder},
        rest_request::RestRequest,
    },
};

use crate::exchange::HmacSha256;

/*----- */
// Poloniex API Authentication
/*----- */
pub struct PoloniexAuthParams;

impl Authenticator for PoloniexAuthParams {
    const SECRET: &'static str = env!("POLONIEX_API_SECRET");
    const KEY: &'static str = env!("POLONIEX_API_KEY");

    #[inline]
    fn generate_signature(request_str: impl Into<String>) -> String {
        let mut mac = HmacSha256::new_from_slice(PoloniexAuthParams::SECRET.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(request_str.into().as_bytes());
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

    // Polnoniex required body to be {"key1": "value1", "key2": "value2"}
    // but serde serialise it to {"key1":"value1","key2":"value2"}, without
    // whitespace. Hence we need this weird replace statements
    pub fn generate_body(&self) -> Result<Option<String>, SocketError> {
        match self.request.body() {
            Some(request_body) => Ok(Some(
                serde_json::to_string(request_body)
                    .map_err(SocketError::Serialise)?
                    .replace(',', ", ")
                    .replace(':', ": "),
            )),
            None => Ok(None),
        }
    }

    pub fn generate_query(&self, request_body: Option<&str>) -> String {
        let body_encoded = match request_body {
            // For requests with fields e.g PoloniexNewOrder
            Some(body) => {
                format!("requestBody={}&signTimestamp={}", body, self.timestamp)
            }
            // For request without fields e.g PoloniexBalance - unit structs
            None => format!("signTimestamp={}", self.timestamp),
        };

        [
            Request::method().as_str(),
            self.request.path().as_ref(),
            body_encoded.as_str(),
        ]
        .join("\n")
    }
}

impl ExchangeRequestBuilder for PoloniexRequestBuilder {
    type AuthParams = PoloniexAuthParams;

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
        let request_query = request_config.generate_query(request_body.as_deref());
        let signature = PoloniexAuthParams::generate_signature(request_query);
        let builder = builder
            .header("Content-Type", "application/json")
            .header("key", PoloniexAuthParams::KEY)
            .header("signature", signature)
            .header("signTimestamp", request_config.timestamp)
            .body(request_body.unwrap_or_default())
            .build()
            .map_err(SocketError::from)?;

        Ok(builder)
    }
}
