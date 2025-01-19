use std::{collections::HashMap, sync::Arc};

use futures::{future::Either, stream::FuturesUnordered, StreamExt};

use rotom_data::{
    exchange::PublicHttpConnector, model::ticker_info::TickerInfo,
    shared::subscription_models::Instrument, streams::builder::single::ExchangeChannel,
    AssetFormatted,
};
use tokio::sync::mpsc;
use tracing::error;

use crate::{
    exchange::{consume_account_data_stream, ExecutionClient},
    execution_manager::request::ExecutionRequestFuture,
    model::{execution_request::ExecutionRequest, execution_response::ExecutionResponse},
};

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

        loop {
            // Get next order out of FuturesUnordered
            let next_open_response = if inflight_opens.is_empty() {
                Either::Left(std::future::pending())
            } else {
                Either::Right(inflight_opens.select_next_some())
            };

            tokio::select! {
                /*----- Handle Execution Requests from Traders ----- */
                Some(request) = self.execution_request_channel.rx.recv() => {
                    match request {
                        ExecutionRequest::Open(request) => {
                            inflight_opens.push(ExecutionRequestFuture::new(
                                    self.execution_client.open_order(request.clone()), //todo make input a clone
                                    self.request_timeout,
                                    request.clone(),
                                ));
                        }
                        ExecutionRequest::Cancel(_request) => {}
                        ExecutionRequest::CancelAll(_request) => {}
                        ExecutionRequest::Transfer(_request) => {}
                    }

                }

                /*----- Process Execution Responses from Exchange ----- */
                Some(execution_response) = self.execution_response_rx.recv() => {
                    if let Err(error) = self.execution_response_tx.send(execution_response) {
                        error!(
                            message = "Error encountered while trying to send back ExecutionResponse to oms",
                            error = %error,
                            exchange =  %Exchange::CLIENT
                        )
                    }
                }

                /*----- Check Results of the FuturesUnordered ----- */
                open_response = next_open_response => {
                    // println!("### Open order ### \n {:#?}", open_response);

                    // When checking http reponse, we only cared if it error. If it
                    // is successful, we would see it come up in the stream
                    if let Err(error) = open_response {
                        println!("{:#?}", error); // todo
                    }
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
