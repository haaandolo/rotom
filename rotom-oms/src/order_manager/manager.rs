use rotom_data::shared::subscription_models::ExchangeId;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::{
    error::OrderManagmentSystemError,
    execution_manager::builder::TraderId,
    model::{execution_response::ExecutionResponse, execution_request::ExecutionRequest, ClientOrderId},
    portfolio::position2::Position2,
};

use super::balance_builder::BalanceMap;

/*----- */
// OMS
/*----- */
#[derive(Debug)]
pub struct OrderManagementSystem {
    balances: BalanceMap,
    open_positions: HashMap<(TraderId, ClientOrderId), Position2>,
    // Receieve ExecutionRequests sent by traders
    execution_request_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
    // Send ExecutionRequests to corresponding ExecutionManager
    execution_manager_txs: HashMap<ExchangeId, mpsc::UnboundedSender<ExecutionRequest>>,
    // Receive ExecutionResponse from ExecutionManger
    execution_response_rx: mpsc::UnboundedReceiver<ExecutionResponse>,
    // Send ExecutionResponses back to corresponding Traders
    execution_response_txs: HashMap<TraderId, mpsc::UnboundedSender<ExecutionResponse>>,
}

impl OrderManagementSystem {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(request) = self.execution_request_rx.recv() => {
                    println!("### Request ### \n {:#?}", request);
                    // match self.execution_manager_txs.get(&request) {
                    //     Some(execution_tx) => {},
                    //     None => {}
                    // }

                },
                Some(response) = self.execution_response_rx.recv() => {
                    println!("### Response ### \n {:#?}", response);
                },
                else => {
                    // This handles the case where both channels are closed
                    println!("Both channels are closed. Exiting loop.");
                    break;
                }

            }
        }
    }
}

/*----- */
// OMS Builder
/*----- */
#[derive(Default)]
pub struct OrderManagementSystemBuilder {
    balances: Option<BalanceMap>,
    execution_request_rx: Option<mpsc::UnboundedReceiver<ExecutionRequest>>,
    execution_manager_txs: Option<HashMap<ExchangeId, mpsc::UnboundedSender<ExecutionRequest>>>,
    execution_response_rx: Option<mpsc::UnboundedReceiver<ExecutionResponse>>,
    execution_response_txs: Option<HashMap<TraderId, mpsc::UnboundedSender<ExecutionResponse>>>,
}

impl OrderManagementSystemBuilder {
    pub fn balances(self, balances: BalanceMap) -> Self {
        Self {
            balances: Some(balances),
            ..self
        }
    }

    pub fn execution_request_rx(
        self,
        execution_request_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
    ) -> Self {
        Self {
            execution_request_rx: Some(execution_request_rx),
            ..self
        }
    }

    pub fn execution_manager_txs(
        self,
        execution_manager_txs: HashMap<ExchangeId, mpsc::UnboundedSender<ExecutionRequest>>,
    ) -> Self {
        Self {
            execution_manager_txs: Some(execution_manager_txs),
            ..self
        }
    }

    pub fn execution_response_rx(
        self,
        execution_response_rx: mpsc::UnboundedReceiver<ExecutionResponse>,
    ) -> Self {
        Self {
            execution_response_rx: Some(execution_response_rx),
            ..self
        }
    }

    pub fn execution_response_txs(
        self,
        execution_response_txs: HashMap<TraderId, mpsc::UnboundedSender<ExecutionResponse>>,
    ) -> Self {
        Self {
            execution_response_txs: Some(execution_response_txs),
            ..self
        }
    }

    pub fn build(self) -> Result<OrderManagementSystem, OrderManagmentSystemError> {
        Ok(OrderManagementSystem {
            balances: self
                .balances
                .ok_or(OrderManagmentSystemError::BuilderIncomplete("balances"))?,
            open_positions: HashMap::with_capacity(100),
            execution_request_rx: self.execution_request_rx.ok_or(
                OrderManagmentSystemError::BuilderIncomplete("execution_request_rx"),
            )?,
            execution_manager_txs: self.execution_manager_txs.ok_or(
                OrderManagmentSystemError::BuilderIncomplete("execution_manager_txs"),
            )?,
            execution_response_rx: self.execution_response_rx.ok_or(
                OrderManagmentSystemError::BuilderIncomplete("execution_manager_txs"),
            )?,
            execution_response_txs: self.execution_response_txs.ok_or(
                OrderManagmentSystemError::BuilderIncomplete("execution_responses_txs"),
            )?,
        })
    }
}
