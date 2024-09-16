use async_trait::async_trait;
use futures::StreamExt;
use rotom_data::error::SocketError;
use rotom_data::protocols::http::client::RestClient;
use rotom_data::protocols::http::http_parser::StandardHttpParser;
use rotom_data::protocols::ws::connect;
use rotom_data::protocols::ws::ws_parser::StreamParser;
use rotom_data::protocols::ws::ws_parser::WebSocketParser;
use rotom_data::protocols::ws::WsRead;
use serde_json::Value;

use crate::execution::exchange::binance::requests::user_data::BinanceUserData;
use crate::execution::exchange::binance::requests::wallet_transfer::BinanceWalletTransfer;
use crate::execution::ExecutionClient2;
use crate::execution::ExecutionId;
use crate::portfolio::OrderEvent;

use super::auth::BinanceAuthParams;
use super::requests::request_builder::BinanceRequest;

// const BINANCE_PRIVATE_ENDPOINT: &str = "wss://ws-api.binance.com:443/ws-api/v3";

#[derive(Debug)]
pub struct BinanceExecution {
    pub user_ws: WsRead,
    pub http_client: reqwest::Client,
    pub test_client: RestClient<StandardHttpParser>,
}

#[async_trait]
impl ExecutionClient2 for BinanceExecution {
    const CLIENT: ExecutionId = ExecutionId::Binance;

    async fn init() -> Result<Self, SocketError> {
        let http_client = reqwest::Client::new();
        let test_client = RestClient::new("https://api.binance.com", StandardHttpParser);

        // listening key
        let listening_key_endpoint = "https://api.binance.com/api/v3/userDataStream";
        let listening_key_res = http_client
            .clone()
            .post(listening_key_endpoint)
            .header("X-MBX-APIKEY", BinanceAuthParams::KEY)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();

        // Spin up listening ws
        let listening_ws = "wss://stream.binance.com:9443/ws/";
        let listening_key = listening_key_res["listenKey"].as_str().unwrap();
        let listening_url = format!("{}{}", listening_ws, listening_key);
        let ws = connect(listening_url).await?;
        let (_, user_ws) = ws.split();

        Ok(BinanceExecution {
            user_ws,
            test_client,
            http_client,
        })
    }

    // Opens order for a single asset
    async fn open_order(&self, open_requests: OrderEvent) {
        let new_order_query = BinanceRequest::new_order(&open_requests).unwrap();
        let res = self.test_client.execute(new_order_query).await.unwrap();
        println!("----- open order: {:#?}", res);
    }

    // Cancels order for a single asset
    async fn cancel_order(&self, orig_client_order_id: String, symbol: String) {
        let cancel_request = BinanceRequest::cancel_order(orig_client_order_id, symbol).unwrap();
        let res = self.test_client.execute(cancel_request).await.unwrap();
        println!("----- cancel order: {:#?}", res);
    }

    // Cancel all orders for a given asset
    async fn cancel_order_all(&self, symbol: String) {
        let cancel_request = BinanceRequest::cancel_order_all(symbol).unwrap();
        let res = self.test_client.execute(cancel_request).await.unwrap();
        println!("----- cancel order all: {:#?}", res);
    }

    // Wallet transfers
    async fn wallet_transfer(&self, coin: String, wallet_address: String) {
        let transfer_request = BinanceWalletTransfer::new(coin, wallet_address).unwrap();
        let res = self.test_client.execute(transfer_request).await.unwrap();
        println!("----- wallet transfer: {:#?}", res);
    }

    async fn receive_reponses(mut self) {
        while let Some(msg) = self.user_ws.next().await {
            let msg_de = WebSocketParser::parse::<BinanceUserData>(msg);
            println!("{:#?}", msg_de);
        }
    }
}
