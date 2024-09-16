use std::borrow::Cow;

use rotom_data::protocols::http::rest_request::RestRequest;
use serde::Deserialize;

/*----- */
// Binance Listening Key
/*----- */
pub struct BinanceListeningKey;

impl RestRequest for BinanceListeningKey {
    type Response = BinanceListeningKeyResponse;
    type QueryParams = ();
    type Body = ();

    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed("/api/v3/userDataStream")
    }

    fn method() -> reqwest::Method {
        reqwest::Method::POST
    }
}

/*----- */
// Binance Listening Key Response
/*----- */
#[derive(Debug, Deserialize)]
pub struct BinanceListeningKeyResponse {
    #[serde(alias = "listenKey")]
    pub listen_key: String,
}
