use std::{collections::HashMap, sync::Arc};

use futures::{future::Either, stream::FuturesUnordered, StreamExt};

use rotom_data::{
    exchange::PublicHttpConnector, model::ticker_info::TickerInfo,
    shared::subscription_models::Instrument, streams::builder::single::ExchangeChannel,
    AssetFormatted,
};
use tokio::sync::mpsc;
use tracing::{debug, error};

use crate::{
    exchange::{consume_account_data_stream, ExecutionClient},
    execution_manager::request::ExecutionRequestFuture,
    model::{execution_request::ExecutionRequest, execution_response::ExecutionResponse},
};

const MAX_RETRIES: u8 = 5;

/*----- */
// Execution Manager
/*----- */
#[derive(Debug)]
pub struct ExecutionManager<Exchange>
where
    Exchange: ExecutionClient,
{
    execution_client: Arc<Exchange>,
    execution_response_tx: mpsc::UnboundedSender<ExecutionResponse>,
    execution_response_rx: mpsc::UnboundedReceiver<ExecutionResponse>,
    ticker_info: HashMap<Instrument, TickerInfo>,
    pub execution_request_channel: ExchangeChannel<ExecutionRequest>,
    request_timeout: std::time::Duration,
}

impl<Exchange> ExecutionManager<Exchange>
where
    Exchange: ExecutionClient + 'static,
    Exchange::PublicData: PublicHttpConnector,
{
    pub async fn init(
        execution_response_tx: mpsc::UnboundedSender<ExecutionResponse>,
        instruments: Vec<Instrument>,
    ) -> Self {
        // Init account data
        let (account_data_tx, account_data_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_stream::<Exchange>(account_data_tx));

        // Init ticker info
        let mut ticker_info = HashMap::with_capacity(instruments.len());

        let ticker_info_futures = instruments
            .into_iter()
            .map(|instrument| {
                ticker_info.insert(instrument.clone(), TickerInfo::default());
                Exchange::PublicData::get_ticker_info(instrument)
            })
            .collect::<Vec<_>>();

        let ticker_info_results = futures::future::try_join_all(ticker_info_futures)
            .await
            .expect("Could not retrieve ticker info");

        for ticker_info_result in ticker_info_results.into_iter() {
            let ticker: TickerInfo = ticker_info_result.into();
            let instrument = ticker_info.iter().find_map(|(key, _)| {
                let formatted_instrument = AssetFormatted::from((&Exchange::CLIENT, key));
                match formatted_instrument.0 == ticker.symbol {
                    true => Some(key.to_owned()),
                    false => None,
                }
            });
            ticker_info.insert(instrument.unwrap(), ticker); // unwrap should never fail
        }

        Self {
            execution_client: Arc::new(Exchange::new()),
            execution_response_tx,
            execution_response_rx: account_data_rx,
            ticker_info,
            execution_request_channel: ExchangeChannel::default(),
            request_timeout: std::time::Duration::from_millis(500), // todo: make exchange specific?
        }
    }

    pub async fn run(mut self) {
        // Init FuturesUnordered
        let mut inflight_opens = FuturesUnordered::new();
        let mut inflight_cancels = FuturesUnordered::new();
        let mut inflight_cancel_alls = FuturesUnordered::new();
        let mut inflight_transfers = FuturesUnordered::new();

        loop {
            // Get next order out of FuturesUnordered
            let next_open_response = if inflight_opens.is_empty() {
                Either::Left(std::future::pending())
            } else {
                Either::Right(inflight_opens.select_next_some())
            };

            let next_cancel_response = if inflight_cancels.is_empty() {
                Either::Left(std::future::pending())
            } else {
                Either::Right(inflight_cancels.select_next_some())
            };

            let next_cancel_all_response = if inflight_cancel_alls.is_empty() {
                Either::Left(std::future::pending())
            } else {
                Either::Right(inflight_cancel_alls.select_next_some())
            };

            let next_transfer_response = if inflight_transfers.is_empty() {
                Either::Left(std::future::pending())
            } else {
                Either::Right(inflight_transfers.select_next_some())
            };

            // Handle Execution Requests from Traders
            tokio::select! {
                Some(request) = self.execution_request_channel.rx.recv() => {
                    match request {
                        ExecutionRequest::Open(mut open_request) => {
                            match self.ticker_info.get(&open_request.request.instrument) {
                                Some(ticker_info) => {
                                    open_request.request.sanitise(&ticker_info.specs);
                                    inflight_opens.push(ExecutionRequestFuture::new(
                                            self.execution_client.open_order(open_request.clone()), //todo make input a clone
                                            self.request_timeout,
                                            open_request.clone(),
                                            0
                                        ));
                                }
                                None => {
                                    error!(
                                        message = "Could not find ticker specs",
                                        payload = format!("{:?}", open_request)
                                    )

                                }
                            }
                        }
                        ExecutionRequest::Cancel(cancel_request) => {
                           inflight_cancels.push(ExecutionRequestFuture::new(
                                self.execution_client.cancel_order(cancel_request.clone()), //todo make input a clone
                                self.request_timeout,
                                cancel_request.clone(),
                                0,
                           ))
                        }
                        ExecutionRequest::CancelAll(cancel_request_all) => {
                           inflight_cancel_alls.push(ExecutionRequestFuture::new(
                                self.execution_client.cancel_order_all(cancel_request_all.clone()), //todo make input a clone
                                self.request_timeout,
                                cancel_request_all.clone(),
                                0
                           ))
                        }
                        ExecutionRequest::Transfer(transfer_request) => {
                           inflight_transfers.push(ExecutionRequestFuture::new(
                                self.execution_client.wallet_transfer(transfer_request.clone()), //todo make input a clone
                                self.request_timeout,
                                transfer_request.clone(),
                                0
                           ))
                        }
                    }
                }

                // Check Results of the FuturesUnordered and if failed resend
                open_response = next_open_response => {
                    if let Err(error) = open_response {
                        // println!("{:#?}", error); // todo
                        debug!(
                            message = "Open order request failed",
                            payload = format!("{:#?}", error)
                        );

                        if error.request_retry_count < MAX_RETRIES {
                            inflight_opens.push(ExecutionRequestFuture::new(
                                    self.execution_client.open_order(error.request.clone()), //todo make input a clone
                                    self.request_timeout,
                                    error.request.clone(),
                                    error.request_retry_count
                                ));
                        } else {
                            let _ = self.execution_response_tx
                                .send(ExecutionResponse::ExecutionError(ExecutionRequest::Open(error.request)));
                        }

                    }
                }

                cancel_response = next_cancel_response => {
                    if let Err(error) = cancel_response {
                        // println!("{:#?}", error); // todo
                        debug!(
                            message = "Cancel request failed",
                            payload = format!("{:#?}", error)
                        );

                        if error.request_retry_count < MAX_RETRIES {
                           inflight_cancels.push(ExecutionRequestFuture::new(
                                self.execution_client.cancel_order(error.request.clone()), //todo make input a clone
                                self.request_timeout,
                                error.request.clone(),
                                error.request_retry_count,
                           ))
                        } else {
                            let _ = self.execution_response_tx
                                .send(ExecutionResponse::ExecutionError(ExecutionRequest::Cancel(error.request)));
                        }
                    }
                }

                cancel_all_response= next_cancel_all_response => {
                    if let Err(error) =  cancel_all_response {
                        // println!("{:#?}", error); // todo
                        debug!(
                            message = "Cancel all request failed",
                            payload = format!("{:#?}", error)
                        );

                        if error.request_retry_count < MAX_RETRIES {
                           inflight_cancel_alls.push(ExecutionRequestFuture::new(
                                self.execution_client.cancel_order_all(error.request.clone()), //todo make input a clone
                                self.request_timeout,
                                error.request.clone(),
                                error.request_retry_count
                           ))
                        } else {
                            let _ = self.execution_response_tx
                                .send(ExecutionResponse::ExecutionError(ExecutionRequest::CancelAll(error.request)));
                        }
                    }
                }

                transfer_response =  next_transfer_response => {
                    if let Err(error) = transfer_response{
                        // println!("{:#?}", error); // todo
                        debug!(
                            message = "Transfer request failed",
                            payload = format!("{:#?}", error)
                        );

                        if error.request_retry_count < MAX_RETRIES {
                           inflight_transfers.push(ExecutionRequestFuture::new(
                                self.execution_client.wallet_transfer(error.request.clone()), //todo make input a clone
                                self.request_timeout,
                                error.request.clone(),
                                error.request_retry_count
                           ))
                        } else {
                            let _ = self.execution_response_tx
                                .send(ExecutionResponse::ExecutionError(ExecutionRequest::Transfer(error.request)));
                        }
                    }
                }

                // Process Execution Responses from Exchange
                Some(execution_response) = self.execution_response_rx.recv() => {
                    if let Err(error) = self.execution_response_tx.send(execution_response) {
                        error!(
                            message = "Error encountered while trying to send back ExecutionResponse to oms",
                            error = %error,
                            exchange =  %Exchange::CLIENT
                        )
                    }
                }

                // Break the loop if both channels are closed
                else => {
                    debug!(
                        message = "All channels in ExecutionManager have closed",
                        action = "Shutting down",
                        exchange = %Exchange::CLIENT,
                    );
                    break
                }
            }
        }
    }
}
