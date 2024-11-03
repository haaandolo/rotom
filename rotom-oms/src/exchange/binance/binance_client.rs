use async_trait::async_trait;
use futures::StreamExt;
use rotom_data::error::SocketError;
use rotom_data::protocols::http::client::RestClient;
use rotom_data::protocols::http::http_parser::StandardHttpParser;
use rotom_data::protocols::ws::connect;
use rotom_data::protocols::ws::ws_parser::StreamParser;
use rotom_data::protocols::ws::ws_parser::WebSocketParser;
use rotom_data::protocols::ws::WsRead;

use crate::exchange::binance::requests::cancel_order::BinanceCancelAllOrder;
use crate::exchange::binance::requests::cancel_order::BinanceCancelOrder;
use crate::exchange::binance::requests::new_order::BinanceNewOrder;
use crate::exchange::binance::requests::user_data::BinanceUserData;
use crate::exchange::binance::requests::wallet_transfer::BinanceWalletTransfer;
use crate::exchange::ExecutionClient2;
use crate::exchange::ExecutionId;
use crate::portfolio::OrderEvent;

use super::request_builder::BinanceRequestBuilder;
use super::requests::cancel_order::BinanceCancelOrderResponse;
use super::requests::listening_key::BinanceListeningKey;
use super::requests::new_order::BinanceNewOrderResponses;
use super::requests::wallet_transfer::BinanceWalletTransferResponse;

/*----- */
// Convinent types
/*----- */
type BinanceRestClient = RestClient<StandardHttpParser, BinanceRequestBuilder>;
const BINANCE_BASE_URL: &str = "https://api.binance.com";
const BINANCE_USER_DATA_WS: &str = "wss://stream.binance.com:9443/ws/";

#[derive(Debug)]
pub struct BinanceExecution {
    pub user_data_ws: WsRead,
    pub http_client: BinanceRestClient,
}

#[async_trait]
impl ExecutionClient2 for BinanceExecution {
    const CLIENT: ExecutionId = ExecutionId::Binance;
    type CancelResponse = BinanceCancelOrderResponse;
    type CancelAllResponse = Vec<BinanceCancelOrderResponse>;
    type NewOrderResponse = BinanceNewOrderResponses;
    type WalletTransferResponse = BinanceWalletTransferResponse;

    async fn init() -> Result<Self, SocketError> {
        // Initialise rest client
        let http_client =
            RestClient::new(BINANCE_BASE_URL, StandardHttpParser, BinanceRequestBuilder);

        // Spin up listening ws
        let (response, _) = http_client.execute(BinanceListeningKey).await?;
        let listening_url = format!("{}{}", BINANCE_USER_DATA_WS, response.listen_key);
        let ws = connect(listening_url).await?;
        let (_, user_data_ws) = ws.split();

        Ok(BinanceExecution {
            user_data_ws,
            http_client,
        })
    }

    async fn open_order(
        &self,
        open_requests: OrderEvent,
    ) -> Result<Self::NewOrderResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceNewOrder::new(&open_requests)?)
            .await?;
        Ok(response.0)
    }

    async fn cancel_order(
        &self,
        orig_client_order_id: String,
        symbol: String,
    ) -> Result<Self::CancelResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceCancelOrder::new(orig_client_order_id, symbol)?)
            .await?;
        Ok(response.0)
    }

    async fn cancel_order_all(
        &self,
        symbol: String,
    ) -> Result<Self::CancelAllResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceCancelAllOrder::new(symbol)?)
            .await?;
        Ok(response.0)
    }

    async fn wallet_transfer(
        &self,
        coin: String,
        wallet_address: String,
        network: Option<String>,
        amount: f64,
    ) -> Result<Self::WalletTransferResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceWalletTransfer::new(coin, wallet_address, network, amount)?)
            .await?;
        Ok(response.0)
    }

    async fn receive_responses(mut self) {
        while let Some(msg) = self.user_data_ws.next().await {
            let msg_de = WebSocketParser::parse::<BinanceUserData>(msg);
            println!("{:#?}", msg_de);
        }
    }
}
