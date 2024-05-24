use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream};
use crate::error::CustomErrors;

pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct WebSocketParser;

pub async fn connect(url: &str) -> Result<WebSocket,CustomErrors> {
    let ws = connect_async(url)
        .await
        .map(|(ws, _)| ws)
        .map_err( CustomErrors::WebSocketConnectionError);
    ws
}

