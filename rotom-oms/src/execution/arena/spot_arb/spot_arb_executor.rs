use rotom_data::error::SocketError;
use serde::Deserialize;
use std::fmt::Debug;
use tokio::sync::mpsc;

use crate::exchange::{consume_account_data_ws, ExecutionClient2};

/*----- */
// Convinenent type to combine two account data streams
/*----- */
#[derive(Debug)]
pub enum CombinedUserStreams<UserDataOne, UserDataTwo> {
    ExchangeOne(UserDataOne),
    ExchangeTwo(UserDataTwo),
}

/*----- */
// Spot Arb Executor - combines exchange execution client's for spot arb
/*----- */
#[derive(Debug)]
pub struct SpotArbExecutor<ExchangeOne, ExchangeTwo>
where
    ExchangeOne: ExecutionClient2,
    ExchangeTwo: ExecutionClient2,
{
    pub exchange_one: ExchangeOne,
    pub exchange_two: ExchangeTwo,
    pub combined_stream: mpsc::UnboundedReceiver<
        CombinedUserStreams<
            ExchangeOne::UserDataStreamResponse,
            ExchangeTwo::UserDataStreamResponse,
        >,
    >,
}

impl<ExchangeOne, ExchangeTwo> SpotArbExecutor<ExchangeOne, ExchangeTwo>
where
    ExchangeOne: ExecutionClient2 + 'static,
    ExchangeTwo: ExecutionClient2 + 'static,
    ExchangeOne::UserDataStreamResponse: Send + for<'de> Deserialize<'de> + Debug,
    ExchangeTwo::UserDataStreamResponse: Send + for<'de> Deserialize<'de> + Debug,
{
    pub async fn new() -> Result<SpotArbExecutor<ExchangeOne, ExchangeTwo>, SocketError> {
        // Convert first exchange ws to channel
        let (exchange_one_tx, mut exchange_one_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_ws::<ExchangeOne>(exchange_one_tx));

        // Convert second exchange ws to channel
        let (exchange_two_tx, mut exchange_two_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_ws::<ExchangeTwo>(exchange_two_tx));

        // Combine channels into one
        let (combined_tx, combined_rx) = mpsc::unbounded_channel();
        let combined_tx_cloned = combined_tx.clone();
        tokio::spawn(async move {
            while let Some(message) = exchange_one_rx.recv().await {
                let _ = combined_tx_cloned.send(CombinedUserStreams::ExchangeOne(message));
            }
        });

        tokio::spawn(async move {
            while let Some(message) = exchange_two_rx.recv().await {
                let _ = combined_tx.send(CombinedUserStreams::ExchangeTwo(message));
            }
        });

        // Init exchange http clients
        let exchange_one_http = ExchangeOne::create_http_client()?;
        let exchange_two_http = ExchangeTwo::create_http_client()?;

        Ok(SpotArbExecutor {
            exchange_one: exchange_one_http,
            exchange_two: exchange_two_http,
            combined_stream: combined_rx,
        })
    }
}
