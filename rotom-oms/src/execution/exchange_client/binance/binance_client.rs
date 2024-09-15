use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use futures::SinkExt;
use futures::StreamExt;
use hmac::Mac;
use rotom_data::error::SocketError;
use rotom_data::protocols::ws::connect;
use rotom_data::protocols::ws::ws_parser::StreamParser;
use rotom_data::protocols::ws::ws_parser::WebSocketParser;
use rotom_data::protocols::ws::JoinHandle;
use rotom_data::protocols::ws::WsMessage;
use rotom_data::protocols::ws::WsRead;
use serde_json::json;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tracing_subscriber::fmt::format;

use crate::execution::exchange_client::binance::requests::wallet_transfer::BinanceWalletTransfer;
use crate::execution::exchange_client::Authenticator;
use crate::execution::exchange_client::HmacSha256;
use crate::execution::exchange_client::OrderEventConverter;
use crate::execution::exchange_client::ParamString;
use crate::execution::ExecutionClient2;
use crate::execution::ExecutionId;
use crate::portfolio::OrderEvent;

use super::auth::BinanceAuthParams;
use super::requests::cancel_order::BinanceCancelAllOrder;
use super::requests::cancel_order::BinanceCancelOrder;
use super::requests::new_order::BinanceNewOrderParams;
use super::requests::request_builder::BinanceRequest;
use super::requests::responses::BinanceResponses;

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
            request_tx: send_tx,
            tasks,
        })
    }

    // Opens order for a single asset
    async fn open_order(&self, open_requests: OrderEvent) {
        let _ = self.request_tx.send(WsMessage::Text(
            serde_json::to_string(
                &BinanceRequest::<BinanceNewOrderParams>::new_order(&open_requests).unwrap(), // TODO
            )
            .unwrap(), // TODO
        ));
    }

    // Cancels order for a single asset
    async fn cancel_order(&self, orig_client_order_id: String) {
        let mut binance_cancel_order = BinanceCancelOrder::new(orig_client_order_id);
        let signature = Self::generate_signature(binance_cancel_order.get_query_param());
        binance_cancel_order.params.signature = Some(signature);
        let _ = self.request_tx.send(WsMessage::Text(
            serde_json::to_string(&binance_cancel_order).unwrap(), // TODO: rm unwrap()
        ));
    }

    // Cancel all orders for a given asset
    async fn cancel_order_all(&self, symbol: String) {
        let mut binance_cancel_all_order = BinanceCancelAllOrder::new(symbol);
        let signature = Self::generate_signature(binance_cancel_all_order.get_query_param());
        binance_cancel_all_order.params.signature = Some(signature);
        let _ = self.request_tx.send(WsMessage::Text(
            serde_json::to_string(&binance_cancel_all_order).unwrap(), // TODO: rm unwrap()
        ));
    }

    // Wallet transfers
    async fn wallet_transfer(&self, coin: String, wallet_address: String) {
        let wallet_endpoint = "https://api.binance.com/sapi/v1/capital/withdraw/apply?";
        let client = reqwest::Client::new();

        let wallet_transfer_request = BinanceWalletTransfer::builder()
            .coin(coin)
            .amount(1.01)
            .address(wallet_address)
            .sign()
            .build()
            .unwrap() // TODO
            .query_param();

        let res = client
            .post(format!("{}{}", wallet_endpoint, wallet_transfer_request))
            .header("X-MBX-APIKEY", BinanceAuthParams::KEY)
            .send()
            .await;

        println!("----- WALLET TRANSFER: {:#?}", res);
    }

    async fn receive_reponses(mut self) {
        while let Some(msg) = self.ws_read.next().await {
            let response = WebSocketParser::parse::<BinanceResponses>(msg);
            println!("{:#?}", response);
        }
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

// Bin conn: first
// coin=OP&address=0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f&amount=10.51&timestamp=1726299139895&signature=72a854ad4e5c16d583abbe4eec232ebc57feaa6b2804812aebe548d5a226729e
// coin=OP&address=0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f&amount=10.01&timestamp=1726301515511&signature=86739f2ffd16f9157997b7a1bb54fc53c029c9b7a943e1509fec534a9f069c05

// Bin conn: second
// coin=OP&address=0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f&amount=10.01&timestamp=1726301914294&signature=720e46a12fba7869b5f7f5e5e48921eb9e31906081dfdd908e56b8269c461a50
// coin=OP&address=0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f&amount=10.01&timestamp=1726301914294&signature=720e46a12fba7869b5f7f5e5e48921eb9e31906081dfdd908e56b8269c461a50

// https://api.binance.com/sapi/v1/capital/withdraw/apply?coin=OP&address=0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f&amount=10.01&timestamp=1726301914294&signature=720e46a12fba7869b5f7f5e5e48921eb9e31906081dfdd908e56b8269c461a50
// https://api.binance.com/sapi/v1/capital/withdraw/apply?coin=OP&address=0xc0b2167fc0ff47fe0783ff6e38c0eecc0f784c2f&amount=10.01&timestamp=1726301914294&signature=720e46a12fba7869b5f7f5e5e48921eb9e31906081dfdd908e56b8269c461a50
