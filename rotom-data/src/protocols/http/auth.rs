use hmac::Mac;
use reqwest::Request;

use crate::error::SocketError;

use super::rest_request::RestRequest;

pub trait Authenticator {
    type AuthParams;

    fn generate_signature(request_str: impl Into<String>) -> String;

    fn build_signed_request<Request>(
        builder: reqwest::RequestBuilder,
        request: Request,
    ) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest;
}
