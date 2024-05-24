use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomErrors {
    #[error("WebSocket error: {0}")]
    WebSocketConnectionError(#[from] tokio_tungstenite::tungstenite::Error)
}