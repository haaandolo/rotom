use chrono::Utc;
use rotom_data::protocols::http::request_builder::Authenticator;
use serde::Serialize;

use crate::exchange::poloniex::request_builder::PoloniexAuthParams;

/*----- */
// Poloniex Ws Auth
/*----- */
#[derive(Debug, Default, Serialize)]
pub struct PoloniexWsAuth {
    event: &'static str,
    channel: [&'static str; 1],
    params: PoloniexWsAuthParams,
}

impl PoloniexWsAuth {
    pub fn new() -> Self {
        Self {
            event: "subscribe",
            channel: ["auth"],
            params: PoloniexWsAuthParams::new(),
        }
    }
}

/*----- */
// Poloniex Ws Auth Params
/*----- */
#[derive(Debug, Default, Serialize)]
pub struct PoloniexWsAuthParams {
    key: &'static str,
    #[serde(rename(serialize = "signTimestamp"))]
    sign_timestamp: i64,
    #[serde(rename(serialize = "signatureMethod"))]
    signature_method: &'static str,
    #[serde(rename(serialize = "sigatureVersion"))]
    signature_version: &'static str,
    signature: String,
}

impl PoloniexWsAuthParams {
    pub fn new() -> Self {
        let timestamp = Utc::now().timestamp_millis();
        let signature_string = format!("{}{}{}{}", "GET\n", "/ws", "\nsignTimestamp=", timestamp);
        Self {
            key: PoloniexAuthParams::KEY,
            sign_timestamp: timestamp,
            signature_method: "HmacSHA256",
            signature_version: "2",
            signature: PoloniexAuthParams::generate_signature(signature_string),
        }
    }
}

/*----- */
// Poloniex Ws Auth - subscribe to user order data request
/*----- */
#[derive(Debug, Default, Serialize)]
pub struct PoloniexWsAuthOrderRequest {
    event: &'static str,
    channel: [&'static str; 1],
    symbols: [&'static str; 1],
}

impl PoloniexWsAuthOrderRequest {
    pub fn new() -> Self {
        Self {
            event: "subscribe",
            channel: ["orders"],
            symbols: ["all"],
        }
    }
}

/*----- */
// Poloniex Ws Auth - subscribe to user balance request
/*----- */
#[derive(Debug, Default, Serialize)]
pub struct PoloniexWsAuthBalanceRequest {
    event: &'static str,
    channel: [&'static str; 1],
}

impl PoloniexWsAuthBalanceRequest {
    pub fn new() -> Self {
        Self {
            event: "subscribe",
            channel: ["balances"],
        }
    }
}
