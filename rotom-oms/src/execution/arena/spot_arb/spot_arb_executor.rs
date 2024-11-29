use rotom_data::error::SocketError;
use std::fmt::Debug;
use tokio::sync::mpsc;

use crate::{
    exchange::{consume_account_data_ws, ExecutionClient},
    model::account_data::AccountData,
};

/*----- */
// Spot Arb Executor - combines exchange execution client's for spot arb
/*----- */
#[derive(Debug)]
pub struct SpotArbExecutor<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
{
    pub liquid_exchange: LiquidExchange,
    pub illiquid_exchange: IlliquidExchange,
    pub streams: mpsc::UnboundedReceiver<AccountData>,
}

impl<LiquidExchange, IlliquidExchange> SpotArbExecutor<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient + 'static,
    IlliquidExchange: ExecutionClient + 'static,
{
    pub async fn init() -> Result<SpotArbExecutor<LiquidExchange, IlliquidExchange>, SocketError> {
        // Convert first exchange ws to channel
        let (liquid_exchange_tx, mut liquid_exchange_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_ws::<LiquidExchange>(
            liquid_exchange_tx,
        ));

        // Convert second exchange ws to channel
        let (illiquid_exchange_tx, mut illiquid_exchange_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_ws::<IlliquidExchange>(
            illiquid_exchange_tx,
        ));

        // Combine channels into one
        let (combined_tx, combined_rx) = mpsc::unbounded_channel();
        let combined_tx_cloned = combined_tx.clone();
        tokio::spawn(async move {
            while let Some(message) = liquid_exchange_rx.recv().await {
                let _ = combined_tx_cloned.send(message);
            }
        });

        tokio::spawn(async move {
            while let Some(message) = illiquid_exchange_rx.recv().await {
                let _ = combined_tx.send(message);
            }
        });

        // Init exchange http clients
        let liquid_exchange_http = LiquidExchange::create_http_client()?;
        let illiquid_exchange_http = IlliquidExchange::create_http_client()?;

        Ok(SpotArbExecutor {
            liquid_exchange: liquid_exchange_http,
            illiquid_exchange: illiquid_exchange_http,
            streams: combined_rx,
        })
    }
}
