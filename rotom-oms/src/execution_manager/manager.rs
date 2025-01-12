use std::{collections::HashMap, sync::Arc};

use futures::{future::Either, stream::FuturesUnordered, StreamExt};
use rotom_data::{error::SocketError, streams::builder::single::ExchangeChannel};
use tokio::sync::mpsc;
use tracing::debug;

use crate::{
    exchange::{consume_account_data_stream, ExecutionClient},
    model::{
        account_data::{AccountDataOrder, ExecutionResponse},
        order::{ExecutionManagerSubscribe, ExecutionRequest, OpenOrder},
        ClientOrderId,
    },
};

use super::builder::TraderId;

/*----- */
// TraderMetaData
/*----- */
#[derive(Debug)]
pub struct TraderUpdateTx(pub mpsc::UnboundedSender<ExecutionResponse>);

/*----- */
// Execution Manager
/*----- */
#[derive(Debug)]
pub struct ExecutionManager<Exchange>
where
    Exchange: ExecutionClient,
{
    execution_client: Arc<Exchange>,
    traders: HashMap<TraderId, TraderUpdateTx>,
    orders: HashMap<(TraderId, ClientOrderId), AccountDataOrder>,
    pub execution_request_channel: ExchangeChannel<ExecutionRequest>,
    account_data_rx: mpsc::UnboundedReceiver<ExecutionResponse>,
    request_timeout: std::time::Duration,
}

impl<Exchange> ExecutionManager<Exchange>
where
    Exchange: ExecutionClient + 'static,
{
    pub fn init() -> Self {
        let (account_data_tx, account_data_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_stream::<Exchange>(account_data_tx));

        Self {
            execution_client: Arc::new(Exchange::new()),
            traders: HashMap::new(),
            orders: HashMap::with_capacity(100),
            execution_request_channel: ExchangeChannel::default(),
            account_data_rx,
            request_timeout: std::time::Duration::from_millis(100), // todo: make exchange specific and include in exeution client
        }
    }

    pub async fn run(mut self) {
        let mut inflight_opens = FuturesUnordered::new();
        loop {
            let next_open_response = if inflight_opens.is_empty() {
                Either::Left(std::future::pending::<
                    Result<Exchange::NewOrderResponse, SocketError>,
                >())
            } else {
                Either::Right(inflight_opens.select_next_some())
            };

            tokio::select! {
                // Handle execution requests
                Some(request) = self.execution_request_channel.rx.recv() => {
                    match request {
                        ExecutionRequest::Subscribe(request) => {
                            // Insert trader tx into TraderUpdateTx HashMap
                            self.traders
                                .entry(request.trader_id)
                                .or_insert(TraderUpdateTx(request.execution_response_tx.clone()));

                            // Try send subscribtion success message to trader
                            if let Err(error) = request.execution_response_tx.send(ExecutionResponse::Subscribed(Exchange::CLIENT)) {
                                debug!(message = "Could not subscribe to trader", error = %error);
                            }

                            // Add coin meta to ExecutionManger
                            


                        },
                        ExecutionRequest::Open(request) => {
                            inflight_opens.push(self.execution_client.open_order(request));
                        }
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

                // Process next ExecutionRequest::Open response
                response_open = next_open_response => {
                    println!("##############");
                    println!("Printing result of open order");
                    println!("##############");
                    println!("Open order res: {:#?}", response_open);

                }

                // Break the loop if both channels are closed
                else => {
                    println!("All channels closed, shutting down execution manager"); // todo
                    break;
                }
            }
        }
    }
}
