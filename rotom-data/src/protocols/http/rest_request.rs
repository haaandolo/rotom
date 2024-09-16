use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

const DEFAULT_HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub trait RestRequest {
    // Expected response type if this request was successful.
    type Response: DeserializeOwned;

    // Serialisable query parameters type - use unit struct () if not required for this request.
    type QueryParams: Serialize;

    // Serialisable Body type - use unit struct () if not required for this request.
    type Body: Serialize;

    // Additional [`Url`](url::Url) path to the resource.
    fn path(&self) -> std::borrow::Cow<'static, str>;

    // Http [`reqwest::Method`] of this request.
    fn method() -> reqwest::Method;

    // Optional query parameters for this request.
    fn query_params(&self) -> Option<&Self::QueryParams> {
        None
    }

    // Optional Body for this request.
    fn body(&self) -> Option<&Self::Body> {
        None
    }

    // Http request timeout
    fn timeout() -> Duration {
        DEFAULT_HTTP_REQUEST_TIMEOUT
    }
}
