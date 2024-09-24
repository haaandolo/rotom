use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

const DEFAULT_HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub trait RestRequest {
    type Response: DeserializeOwned;
    type QueryParams: Serialize;
    type Body: Serialize;

    fn path(&self) -> std::borrow::Cow<'static, str>;

    fn method() -> reqwest::Method;

    fn query_params(&self) -> Option<&Self::QueryParams> {
        None
    }

    fn body(&self) -> Option<&Self::Body> {
        None
    }

    fn timeout() -> Duration {
        DEFAULT_HTTP_REQUEST_TIMEOUT
    }
}
