use async_trait::async_trait;
use futures::SinkExt;
use futures::StreamExt;
use rotom_data::error::SocketError;
use rotom_data::protocols::ws::connect;
use rotom_data::protocols::ws::ws_parser::StreamParser;
use rotom_data::protocols::ws::ws_parser::WebSocketParser;
use rotom_data::protocols::ws::JoinHandle;
use rotom_data::protocols::ws::WsMessage;
use rotom_data::protocols::ws::WsRead;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;

use crate::execution::exchange_client::binance::responses::BinanceResponses;
use crate::execution::ExecutionClient2;
use crate::execution::ExecutionId;
use crate::portfolio::OrderEvent;

use super::auth::BinanceAuthParams;
use super::requests::request_builder::BinanceRequest;

const BINANCE_PRIVATE_ENDPOINT: &str = "wss://ws-api.binance.com:443/ws-api/v3";

#[derive(Debug)]
pub struct BinanceExecution {
    pub ws_read: WsRead,
    pub http_client: reqwest::Client,
    pub request_tx: UnboundedSender<WsMessage>,
    pub tasks: Vec<JoinHandle>,
}

#[async_trait]
impl ExecutionClient2 for BinanceExecution {
    const CLIENT: ExecutionId = ExecutionId::Binance;

    async fn init() -> Result<Self, SocketError> {
        // Initalise ws
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
            http_client: reqwest::Client::new(),
            request_tx: send_tx,
            tasks,
        })
    }

    // // Opens order for a single asset
    // async fn open_order(&self, open_requests: OrderEvent) {
    //     let _ = self.request_tx.send(WsMessage::Text(
    //         serde_json::to_string(
    //             &BinanceRequest::new_order(&open_requests).unwrap(), // TODO
    //         )
    //         .unwrap(), // TODO
    //     ));
    // }

    // Opens order for a single asset
    async fn open_order(&self, open_requests: OrderEvent) {
        let new_order_endpoint = "https://api.binance.com/api/v3/order?";
        let new_order_query = BinanceRequest::new_order(&open_requests)
            .unwrap()
            .query_param(); // TODO

        let res = self
            .http_client
            .post(format!("{}{}", new_order_endpoint, new_order_query))
            .header("X-MBX-APIKEY", BinanceAuthParams::KEY)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await;

        println!("----- open order: {:#?}", res);
    }

    // Cancels order for a single asset
    async fn cancel_order(&self, orig_client_order_id: String, symbol: String) {
        let cancel_endpoint = "https://api.binance.com/api/v3/order?";
        let cancel_request = BinanceRequest::cancel_order(orig_client_order_id, symbol)
            .unwrap()
            .query_param(); // TODO

        let res = self
            .http_client
            .delete(format!("{}{}", cancel_endpoint, cancel_request))
            .header("X-MBX-APIKEY", BinanceAuthParams::KEY)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await;

        println!("----- cancel order: {:#?}", res);
    }

    // Cancel all orders for a given asset
    async fn cancel_order_all(&self, symbol: String) {
        let cancel_endpoint = "https://api.binance.com/api/v3/openOrders?";
        let cancel_request = BinanceRequest::cancel_order_all(symbol)
            .unwrap()
            .query_param(); // TODO

        let res = self
            .http_client
            .delete(format!("{}{}", cancel_endpoint, cancel_request))
            .header("X-MBX-APIKEY", BinanceAuthParams::KEY)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await;

        println!("----- cancel order: {:#?}", res);
    }

    // Wallet transfers
    async fn wallet_transfer(&self, coin: String, wallet_address: String) {
        let wallet_endpoint = "https://api.binance.com/sapi/v1/capital/withdraw/apply?";

        let wallet_transfer_request =
            BinanceRequest::wallet_transfer(coin, wallet_address).unwrap(); // TODO

        let res = self
            .http_client
            .post(format!("{}{}", wallet_endpoint, wallet_transfer_request))
            .header("X-MBX-APIKEY", BinanceAuthParams::KEY)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await;

        println!("----- wallet transfer: {:#?}", res);
    }

    async fn receive_reponses(mut self) {
        while let Some(msg) = self.ws_read.next().await {
            let response = WebSocketParser::parse::<BinanceResponses>(msg);
            println!("{:#?}", response);
        }
    }
}
