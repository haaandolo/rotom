use std::fmt::Debug;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest, MaybeTlsStream};

pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsMessage = tokio_tungstenite::tungstenite::Message;

use tracing::debug;

#[derive(Debug)]
pub enum SocketError {
    WebSocket(tokio_tungstenite::tungstenite::Error),
}


/// Connect asynchronously to a [`WebSocket`] server.
pub async fn connect<R>(request: R) -> Result<WebSocket, SocketError>
where
    R: IntoClientRequest + Unpin + Debug,
{
    debug!(?request, "attempting to establish WebSocket connection");
    connect_async(request)
        .await
        .map(|(websocket, _)| websocket)
        .map_err(SocketError::WebSocket)
}
