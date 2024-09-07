use async_trait::async_trait;
use futures::SinkExt;
use futures::StreamExt;
use hmac::Mac;
use rotom_data::error::SocketError;
use rotom_data::protocols::ws::connect;
use rotom_data::protocols::ws::JoinHandle;
use rotom_data::protocols::ws::WsMessage;
use rotom_data::protocols::ws::WsRead;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;

use crate::execution::exchange_client::binance::requests::new_order::BinanceNewOrder;
use crate::execution::exchange_client::Authenticator;
use crate::execution::exchange_client::HmacSha256;
use crate::execution::exchange_client::OrderEventConverter;
use crate::execution::exchange_client::ParamString;
use crate::execution::ExecutionClient2;
use crate::execution::ExecutionId;
use crate::portfolio::OrderEvent;

use super::auth::BinanceAuthParams;

const BINANCE_PRIVATE_ENDPOINT: &str = "wss://ws-api.binance.com:443/ws-api/v3";

#[derive(Debug)]
pub struct BinanceExecution {
    pub ws_read: WsRead,
    pub request_tx: UnboundedSender<WsMessage>,
    pub tasks: Vec<JoinHandle>,
}

#[async_trait]
impl ExecutionClient2 for BinanceExecution {
    const CLIENT: ExecutionId = ExecutionId::Binance;

    async fn init() -> Result<Self, SocketError> {
        let ws = connect(BINANCE_PRIVATE_ENDPOINT).await?;
        let (mut ws_write, ws_read) = ws.split();
        let mut tasks = Vec::new();

        // Spawn ws_write into the ether
        let (send_tx, mut read_rx) = mpsc::unbounded_channel();
        let write_handler = tokio::spawn(async move {
            while let Some(msg) = read_rx.recv().await {
                let _ = ws_write.send(msg).await;
            }
        });
        tasks.push(write_handler);

        Ok(BinanceExecution {
            ws_read,
            request_tx: send_tx,
            tasks,
        })
    }

    async fn open_orders(&self, open_requests: OrderEvent) {
        let mut binance_order = BinanceNewOrder::new(&open_requests);
        let signature = Self::generate_signature(binance_order.get_query_param());
        binance_order.params.signature = Some(signature);
        let _ = self.request_tx.send(WsMessage::Text(
            serde_json::to_string(&binance_order).unwrap(), // TODO: rm unwrap()
        ));
    }
}

impl Authenticator for BinanceExecution {
    type AuthParams = BinanceAuthParams;

    fn generate_signature(request_str: ParamString) -> String {
        let mut mac = HmacSha256::new_from_slice(BinanceAuthParams::SECRET.as_bytes())
            .expect("Could not generate HMAC for Binance");
        mac.update(request_str.0.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}