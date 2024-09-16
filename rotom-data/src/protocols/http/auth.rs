use crate::error::SocketError;

pub trait Authenticator {
    type AuthParams;

    fn build_signed_request(
        builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Request, SocketError>;
}
