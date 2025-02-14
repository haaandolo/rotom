use rotom_data::shared::subscription_models::ExchangeId;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::error;

use crate::{
    error::OrderManagmentSystemError,
    execution_manager::builder::TraderId,
    model::{
        execution_request::ExecutionRequest, execution_response::ExecutionResponse, ClientOrderId,
    },
    portfolio::position2::Position2,
};

use super::maps::{BalanceMap, OrderMap};

/*----- */
// OMS
/*----- */
#[derive(Debug)]
pub struct OrderManagementSystem {
    balances: BalanceMap,
    open_positions: OrderMap,
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
                    // Find ExecutionManager tx and send ExecutionRequest. If not found, shut down program
                    match self.execution_manager_txs.get(&request.get_exchange_id()) {
                        Some(execution_tx) => {
                            if let Err(error) = execution_tx.send(request) {
                                error! {
                                    message = "Error encountered while trying to send ExecutionRequest to ExecutionManager",
                                    error = %error,
                                }
                            }
                        },
                        None => {
                            error! {
                                message = "Could not find tx to send ExecutionRequest to ExecutionManager. Check ExecutionManger initialisation for provided request",
                                payload = format!("{:?}", request),
                                action = "Shutting down entire program"
                            }
                            std::process::exit(1);
                        }
                    }
                },
                Some(response) = self.execution_response_rx.recv() => {
                    println!("### Response ### \n {:#?}", response);
                    match response {
                        ExecutionResponse::Order(order) => {}
                        ExecutionResponse::Balance(balance) => {
                            self.balances.update_balance(&balance);
                        }
                        ExecutionResponse::BalanceVec(balance_vec) => {
                            for balance in balance_vec.iter() {
                                self.balances.update_balance(balance);
                            }
                        }
                        ExecutionResponse::BalanceDelta(balance_delta) => {
                            self.balances.update_balance_delta(&balance_delta);
                        }
                        ExecutionResponse::ExecutionError(error) => {
                            let execution_response_tx = self.execution_response_txs
                                .get(&error.get_trader_id())
                                .expect("Cannot find execution response tx");

                            let _ =  execution_response_tx.send(ExecutionResponse::ExecutionError(error));
                        }

                    }
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
            open_positions: OrderMap(HashMap::with_capacity(100)),
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
