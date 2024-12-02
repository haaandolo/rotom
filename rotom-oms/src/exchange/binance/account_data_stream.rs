use async_trait::async_trait;
use futures::StreamExt;
use rotom_data::{
    error::SocketError,
    protocols::{http::http_parser::StandardHttpParser, ws::connect},
};

use crate::exchange::{AccountDataStream, ExecutionId, UserDataStream};

use super::{
    binance_client::{BinanceRestClient, BINANCE_BASE_URL},
    request_builder::BinanceRequestBuilder,
    requests::{account_data::BinanceAccountEvents, listening_key::BinanceListeningKey},
};

const BINANCE_USER_DATA_WS: &str = "wss://stream.binance.com:9443/ws/";

#[derive(Debug)]
pub struct BinanaceAccountDataStream;

#[async_trait]
impl AccountDataStream for BinanaceAccountDataStream {
    const CLIENT: ExecutionId = ExecutionId::Binance;
    type AccountDataStreamResponse = BinanceAccountEvents;

    async fn init() -> Result<UserDataStream, SocketError> {
        let http_client =
            BinanceRestClient::new(BINANCE_BASE_URL, StandardHttpParser, BinanceRequestBuilder);

        let (response, _) = http_client.execute(BinanceListeningKey).await?;
        let listening_url = format!("{}{}", BINANCE_USER_DATA_WS, response.listen_key);
        let ws = connect(listening_url).await?;
        let (_, user_data_ws) = ws.split();

        Ok(UserDataStream {
            user_data_ws,
            tasks: None,
        })
    }
}
