use std::collections::HashMap;

use rotom_data::shared::subscription_models::ExchangeId;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    exchange::ExecutionClient,
    model::{account_data::AccountData, order::ExecutionRequest},
};

use super::manager::ExecutionManager;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TraderId(pub Uuid);

#[derive(Debug)]
pub struct ExecutionBuilder {
    pub execution_request_tx: HashMap<ExchangeId, mpsc::UnboundedSender<ExecutionRequest>>,
}

impl ExecutionBuilder {
    pub fn add_exchange<Exchange>(
        mut self,
        trader_update_txs: HashMap<TraderId, mpsc::Sender<AccountData>>,
    ) -> Self
    where
        Exchange: ExecutionClient + 'static,
    {
        let execution_manager = ExecutionManager::<Exchange>::init(trader_update_txs);
        self.execution_request_tx.insert(
            Exchange::CLIENT,
            execution_manager.execution_request_tx.clone(),
        );
        self
    }
}

/*
    pub fn add_live<Client>(
        self,
        config: Client::Config,
        request_timeout: Duration,
    ) -> Result<Self, BarterError>
    where
        Client: ExecutionClient + Send + Sync + 'static,
        Client::AccountStream: Send,
    {
        self.add_execution::<Client>(Client::EXCHANGE, config, request_timeout)
    }

    fn add_execution<Client>(
        mut self,
        exchange: ExchangeId,
        config: Client::Config,
        request_timeout: Duration,
    ) -> Result<Self, BarterError>
    where
        Client: ExecutionClient + Send + Sync + 'static,
        Client::AccountStream: Send,
    {
        let instrument_map = generate_execution_instrument_map(self.instruments, exchange)?;

        let (execution_tx, execution_rx) = mpsc_unbounded();

        if self
            .channels
            .insert(exchange, (instrument_map.exchange.key, execution_tx))
            .is_some()
        {
            return Err(BarterError::ExecutionBuilder(format!(
                "ExecutionBuilder does not support duplicate mocked ExecutionManagers: {exchange}"
            )));
        }

        let merged_tx = self.merged_channel.tx.clone();

        self.futures.push(Box::pin(async move {
            // Initialise ExecutionManager
            let (manager, account_stream) = ExecutionManager::init(
                execution_rx.into_stream(),
                request_timeout,
                Arc::new(Client::new(config)),
                AccountEventIndexer::new(Arc::new(instrument_map)),
                STREAM_RECONNECTION_POLICY,
            )
            .await?;

            tokio::spawn(manager.run());
            tokio::spawn(account_stream.forward_to(merged_tx));

            Ok(())
        }));

        Ok(self)
    }

*/
