use std::{collections::HashMap, fmt::Debug};

use rotom_data::{
    exchange::PublicHttpConnector,
    shared::subscription_models::{ExchangeId, Instrument},
};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    exchange::ExecutionClient,
    model::{execution_request::ExecutionRequest, execution_response::ExecutionResponse},
};

use super::manager::ExecutionManager;

#[derive(Debug, Copy, Eq, PartialEq, Hash)]
pub struct TraderId(pub Uuid); // todo: should be in common types crate

impl std::fmt::Display for TraderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Clone for TraderId {
    #[inline]
    fn clone(&self) -> Self {
        *self // Uses Copy instead of actually cloning
    }
}

#[derive(Default)]
pub struct ExecutionBuilder {
    pub execution_request_tx: HashMap<ExchangeId, mpsc::UnboundedSender<ExecutionRequest>>,
}

impl ExecutionBuilder {
    pub async fn add_exchange<Exchange>(
        mut self,
        execution_response_tx: mpsc::UnboundedSender<ExecutionResponse>,
        instruments: Vec<Instrument>,
    ) -> Self
    where
        Exchange: ExecutionClient + Send + Sync + Debug + 'static,
        Exchange::PublicData: PublicHttpConnector,
    {
        // Initialise ExecutionManager
        let execution_manager =
            ExecutionManager::<Exchange>::init(execution_response_tx, instruments).await;

        println!(
            "### ExecutionManger Builder ### \n {:#?}",
            execution_manager
        );

        // Add ExecutionManager ExecutionRequest tx to hashmap for traders to use
        self.execution_request_tx.insert(
            Exchange::CLIENT,
            execution_manager.execution_request_channel.tx.clone(),
        );

        // Tokio spawn ExecutionManager into the ether
        tokio::spawn(async move { execution_manager.run().await });

        self
    }

    pub fn build(self) -> HashMap<ExchangeId, mpsc::UnboundedSender<ExecutionRequest>> {
        self.execution_request_tx
    }
}
