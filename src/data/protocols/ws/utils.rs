use futures::{SinkExt, StreamExt};
use tokio::{sync::mpsc::UnboundedSender, time::{sleep, Duration}};
use tokio_tungstenite::tungstenite::error::ProtocolError;

use crate::data::{protocols::ws::WsMessage, shared::orderbook::event::Event};
use super::{PingInterval,  WebSocketClient, WsError, WsWrite};

pub const START_RECONNECTION_BACKOFF_MS: u64 = 125;

pub async fn schedule_pings_to_exchange(mut ws_write: WsWrite, ping_interval: PingInterval) {
    loop {
        sleep(Duration::from_secs(ping_interval.time)).await;
        ws_write
            .send(WsMessage::Text(ping_interval.message.to_string()))
            .await
            .expect("Failed to send ping to ws");
    }
}

pub fn is_websocket_disconnected(error: &WsError) -> bool {
    matches!(
        error,
        WsError::ConnectionClosed
            | WsError::AlreadyClosed
            | WsError::Io(_)
            | WsError::Protocol(ProtocolError::SendAfterClosing)
            | WsError::Protocol(ProtocolError::ResetWithoutClosingHandshake) // | WsError::Protocol(_)
    )
}

pub async fn try_connect(mut ws_client: WebSocketClient, exchange_tx: UnboundedSender<Event>) {
    let mut _connection_attempt: u32 = 0;
    let mut _backoff_ms: u64 = START_RECONNECTION_BACKOFF_MS;

    loop {
        _connection_attempt += 1;
        _backoff_ms *= 2;

        // Attempt to connect to the stream
        let mut stream = match ws_client.connect().await {
            Ok(stream) => {
                _connection_attempt = 0;
                _backoff_ms = START_RECONNECTION_BACKOFF_MS;
                stream
            }
            Err(error) => {
                if _connection_attempt == 1 {
                    panic!("First connection attemp failed with error: {:#?}", error);
                // CHANGE THIS PANIC
                } else {
                    continue;
                }
            }
        };

        // Read from stream and send via channel, but if error occurs, attempt reconnection
        while let Some(message) = stream.ws_read.next().await {
            match message {
                Ok(message) => exchange_tx.send(message).expect("Failed to send message"),
                Err(error) => (),
                // Ok(message) => {
                //     match message {
                //         WsMessage::Text(text) => exchange_tx.send(text).expect("Failed to send message"),
                //         _ => ()
                //     }
                // }
                // Err(error) => {
                //     if is_websocket_disconnected(&error) { // SHOULD CHANGE THIS TO SEE WHAT ERRORS ACTUAL MEANS RECONNECTING
                //         stream.cancel_running_tasks()
                //     }
                // } // ADD Error for if sequence is broken then have to restart
            }
        }

        // Wait a certain ms before trying to reconnect
        sleep(Duration::from_millis(_backoff_ms)).await;
    }
}