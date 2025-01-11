use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::{
    exchange::{consume_account_data_stream, ExecutionClient},
    model::{
        account_data::ExecutionResponse,
        order::{ExecutionManagerSubscribe, ExecutionRequest},
    },
};

use super::builder::TraderId;

/*----- */
// TraderMetaData
/*----- */
#[derive(Debug)]
pub struct TraderMetaData {
    pub update_tx: mpsc::UnboundedSender<ExecutionResponse>,
    pub orders: Vec<ExecutionResponse>,
}

impl TraderMetaData {
    pub fn new(update_tx: mpsc::UnboundedSender<ExecutionResponse>) -> Self {
        Self {
            update_tx,
            orders: Vec::with_capacity(5),
        }
    }
}

/*----- */
// Execution Manager
/*----- */
#[derive(Debug)]
pub struct ExecutionManager<Exchange>
where
    Exchange: ExecutionClient,
{
    pub execution_client: Exchange,
    pub traders: HashMap<TraderId, TraderMetaData>,
    pub execution_request_tx: mpsc::UnboundedSender<ExecutionRequest>,
    execution_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
    account_data_rx: mpsc::UnboundedReceiver<ExecutionResponse>,
    pub request_timeout: std::time::Duration,
}

impl<Exchange> ExecutionManager<Exchange>
where
    Exchange: ExecutionClient + 'static,
{
    pub fn init() -> Self {
        let (account_data_tx, account_data_rx) = mpsc::unbounded_channel();
        let (execution_request_tx, execution_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_stream::<Exchange>(account_data_tx));

        Self {
            execution_client: Exchange::new(),
            traders: HashMap::new(),
            execution_request_tx,
            execution_rx,
            account_data_rx,
            request_timeout: std::time::Duration::from_millis(100), // todo: make exchange specific and include in exeution client
        }
    }

    fn handle_subscription(&mut self, subscription_request: ExecutionManagerSubscribe) {
        let execution_response_tx = subscription_request.execution_response_tx;

        self.traders
            .entry(subscription_request.trader_id)
            .or_insert(TraderMetaData::new(execution_response_tx.clone()));

        let _ = execution_response_tx.send(ExecutionResponse::Subscribed(Exchange::CLIENT));
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // Handle execution requests
                // Some(request) = self.execution_rx.recv() => {
                Some(request) = self.execution_rx.recv() => {
                    println!("##############");
                    println!("In execution manager");
                    println!("##############");
                    println!("Request: {:#?}", request);

                    match request {
                        ExecutionRequest::Subscribe(request) => self.handle_subscription(request),
                        ExecutionRequest::Open(_request) => {}
                        ExecutionRequest::Cancel(_request) => {}
                        ExecutionRequest::CancelAll(_request) => {}
                        ExecutionRequest::Transfer(_request) => {}
                    }

                }

                // Handle account data updates
                Some(account_data) = self.account_data_rx.recv() => {
                    println!("##############");
                    println!("In execution manager");
                    println!("##############");
                    println!("Account Data: {:#?}", account_data);
                }

                // Break the loop if both channels are closed
                else => {
                    println!("All channels closed, shutting down execution manager");
                    break;
                }
            }
        }
    }
}
