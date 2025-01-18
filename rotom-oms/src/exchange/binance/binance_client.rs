use async_trait::async_trait;
use futures::StreamExt;
use rotom_data::error::SocketError;
use rotom_data::exchange::binance::BinanceSpotPublicData;
use rotom_data::protocols::http::client::RestClient;
use rotom_data::protocols::http::http_parser::StandardHttpParser;
use rotom_data::protocols::ws::connect;
use rotom_data::shared::subscription_models::ExchangeId;

use crate::exchange::binance::requests::cancel_order::BinanceCancelOrder;
use crate::exchange::binance::requests::new_order::BinanceNewOrder;
use crate::exchange::binance::requests::wallet_transfer::BinanceWalletTransfer;
use crate::exchange::AccountDataWebsocket;
use crate::exchange::ExecutionClient;
use crate::model::account_data::AccountDataBalance;
use crate::model::execution_request::CancelOrder;
use crate::model::execution_request::OpenOrder;
use crate::model::execution_request::WalletTransfer;

use super::request_builder::BinanceRequestBuilder;
use super::requests::account_data::BinanceAccountEvents;
use super::requests::balance::BinanceBalance;
use super::requests::cancel_order::BinanceCancelAllOrder;
use super::requests::cancel_order::BinanceCancelOrderResponse;
use super::requests::listening_key::BinanceListeningKey;
use super::requests::new_order::BinanceNewOrderResponses;
use super::requests::wallet_transfer::BinanceWalletTransferResponse;

/*----- */
// Convinent types
/*----- */
pub type BinanceRestClient = RestClient<StandardHttpParser, BinanceRequestBuilder>;
pub const BINANCE_BASE_URL: &str = "https://api.binance.com";
const BINANCE_USER_DATA_WS: &str = "wss://stream.binance.com:9443/ws/";

#[derive(Debug)]
pub struct BinanceExecution {
    pub http_client: BinanceRestClient,
}

#[async_trait]
impl ExecutionClient for BinanceExecution {
    const CLIENT: ExchangeId = ExchangeId::BinanceSpot;

    type PublicData = BinanceSpotPublicData;
    type CancelResponse = BinanceCancelOrderResponse;
    type CancelAllResponse = Vec<BinanceCancelOrderResponse>;
    type NewOrderResponse = BinanceNewOrderResponses;
    type WalletTransferResponse = BinanceWalletTransferResponse;
    type AccountDataStreamResponse = BinanceAccountEvents;

    async fn init() -> Result<AccountDataWebsocket, SocketError> {
        let http_client =
            BinanceRestClient::new(BINANCE_BASE_URL, StandardHttpParser, BinanceRequestBuilder);

        let (response, _) = http_client.execute(BinanceListeningKey).await?;
        let listening_url = format!("{}{}", BINANCE_USER_DATA_WS, response.listen_key);
        let ws = connect(listening_url).await?;
        let (_, user_data_ws) = ws.split();

        Ok(AccountDataWebsocket {
            user_data_ws,
            tasks: None,
        })
    }

    fn new() -> Self {
        let http_client =
            BinanceRestClient::new(BINANCE_BASE_URL, StandardHttpParser, BinanceRequestBuilder);
        BinanceExecution { http_client }
    }

    async fn open_order(
        &self,
        open_requests: OpenOrder,
    ) -> Result<Self::NewOrderResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceNewOrder::new(open_requests)?)
            .await?;
        Ok(response.0)
    }

    async fn cancel_order(
        &self,
        cancel_request: CancelOrder,
    ) -> Result<Self::CancelResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceCancelOrder::new(
                cancel_request.client_order_id,
                cancel_request.symbol,
            )?)
            .await?;
        Ok(response.0)
    }

    async fn cancel_order_all(
        &self,
        cancel_request: CancelOrder,
    ) -> Result<Self::CancelAllResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceCancelAllOrder::new(cancel_request.symbol)?)
            .await?;
        Ok(response.0)
    }

    async fn wallet_transfer(
        &self,
        wallet_transfer_request: WalletTransfer,
    ) -> Result<Self::WalletTransferResponse, SocketError> {
        let response = self
            .http_client
            .execute(BinanceWalletTransfer::new(
                wallet_transfer_request.coin,
                wallet_transfer_request.wallet_address,
                wallet_transfer_request.network,
                wallet_transfer_request.amount,
            )?)
            .await?;
        Ok(response.0)
    }

    async fn get_balances() -> Result<Vec<AccountDataBalance>, SocketError> {
        let http_client =
            RestClient::new(BINANCE_BASE_URL, StandardHttpParser, BinanceRequestBuilder);
        let response = http_client.execute(BinanceBalance::new()?).await?;
        let account_data: Vec<AccountDataBalance> = response.0.into();
        Ok(account_data)
    }
}
