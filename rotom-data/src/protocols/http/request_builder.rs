use crate::error::SocketError;

use super::rest_request::RestRequest;

/*----- */
// Authenticator
/*----- */
pub trait Authenticator {
    const KEY: &'static str;
    const SECRET: &'static str;

    fn generate_signature(request_str: impl Into<String>) -> String;
}

/*----- */
// ExchangeRequestBuilder
/*----- */
pub trait ExchangeRequestBuilder {
    type AuthParams: Authenticator;

    fn build_signed_request<Request>(
        builder: reqwest::RequestBuilder,
        request: Request,
    ) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest;
}
