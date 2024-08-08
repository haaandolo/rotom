use thiserror::Error;

use super::{protocols::ws::WsError, shared::subscription_models::ExchangeId};

/*----- */
// WebSocketError
/*----- */
#[derive(Debug, Error)]
pub enum SocketError {
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Parsing error")]
    ParsingError(String),

    #[error("Deserialising JSON error: {error} for payload: {payload}")]
    Deserialise {
        error: serde_json::Error,
        payload: String,
    },

    #[error("Deserialising JSON error: {error} for binary payload: {payload:?}")]
    DeserialiseBinary {
        error: serde_json::Error,
        payload: Vec<u8>,
    },

    #[error("Serialising JSON error: {0}")]
    Serialise(serde_json::Error),

    #[error("Unable to find orderbook for {symbol}")]
    OrderBookFindError { symbol: String },

    #[error("Transformer returned None")]
    TransformerNone,

    #[error("ExchangeStream terminated with closing frame: {0}")]
    Terminated(String),

    #[error("error subscribing to resources over the socket: {0}")]
    Subscribe(String),

    #[error("HTTP error: {0}")]
    Http(reqwest::Error),

    #[error("Init method failed for ticker some tickers")]
    Init,

    #[error("Could not retrieve tick size for {base}{quote}, {exchange}")]
    TickSizeError {
        base: String,
        quote: String,
        exchange: ExchangeId,
    },

    // Terminal errors
    #[error("{symbol} got InvalidSequence, first_update_id {first_update_id} does not follow on from the prev_last_update_id {prev_last_update_id}")]
    InvalidSequence {
        symbol: String,
        prev_last_update_id: u64,
        first_update_id: u64,
    },

    #[error("WebSocket disconnected: {error}")]
    WebSocketDisconnected { error: WsError },
}

impl SocketError {
    #[allow(clippy::match_like_matches_macro)]
    pub fn is_terminal(&self) -> bool {
        match self {
            SocketError::InvalidSequence { .. } => true,
            SocketError::WebSocketDisconnected { .. } => true,
            _ => false,
        }
    }
}

/*----- */
// ThisError example
/*----- */
// use thiserror::Error;
//
// #[derive(Debug, Error)]
// pub enum AppError {
//     #[error("Didn't get a query string")]
//     MissingQuery,
//     #[error("Didn't get a file name")]
//     MissingFilename,
//     #[error("Could not load config")]
//     ConfigLoad {
//         #[from]
//         source: io::Error,
//     },
// }
//
// The above ConfigLoad error will produce this error:
// Could not load config: Os { code: 2, kind: NotFound, message: "No such file or directory" }
