use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

#[derive(Debug, Error)]
pub enum NetworkErrors {
    #[error("WebSocket error: {0}")]
    WebSocketConnectionError(#[from] tokio_tungstenite::tungstenite::Error)
}