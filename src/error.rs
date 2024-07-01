use thiserror::Error;

// pub type Result<T> = core::result::Result<T, SocketError>;
// pub type Error = Box<dyn std::error::Error>;

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

    #[error("Error to consume: {0}")]
    ConsumeError(String),

    #[error("ExchangeStream terminated with closing frame: {0}")]
    Terminated(String),

    #[error("error subscribing to resources over the socket: {0}")]
    Subscribe(String),
}


#[derive(Debug, Error)]
pub enum StandardError {
    #[error("Std Error: {0}")]
    StdError(#[from] Box<dyn std::error::Error>)
}