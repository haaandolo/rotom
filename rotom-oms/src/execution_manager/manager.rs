use std::{collections::HashMap, sync::Arc};

use futures::{future::Either, stream::FuturesUnordered, StreamExt};
use rotom_data::{
    error::SocketError, exchange::PublicHttpConnector, model::ticker_info::TickerInfo,
    shared::subscription_models::Instrument, streams::builder::single::ExchangeChannel,
    AssetFormatted,
};
use tokio::sync::mpsc;
use tracing::{debug, error};

use crate::{
    exchange::{consume_account_data_stream, ExecutionClient},
    execution_manager::request::ExecutionRequestFuture,
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
    ticker_info: HashMap<Instrument, TickerInfo>,
    pub execution_request_channel: ExchangeChannel<ExecutionRequest>,
    account_data_rx: mpsc::UnboundedReceiver<ExecutionResponse>,
    request_timeout: std::time::Duration,
}

impl<Exchange> ExecutionManager<Exchange>
where
    Exchange: ExecutionClient + 'static,
    Exchange::PublicData: PublicHttpConnector,
{
    pub fn init() -> Self {
        let (account_data_tx, account_data_rx) = mpsc::unbounded_channel();
        tokio::spawn(consume_account_data_stream::<Exchange>(account_data_tx));

        Self {
            execution_client: Arc::new(Exchange::new()),
            traders: HashMap::new(),
            orders: HashMap::with_capacity(100),
            ticker_info: HashMap::with_capacity(100),
            execution_request_channel: ExchangeChannel::default(),
            account_data_rx,
            request_timeout: std::time::Duration::from_millis(200), // todo: make exchange specific?
        }
    }

    pub async fn run(mut self) {
        // Init FuturesUnordered
        let mut inflight_opens = FuturesUnordered::new();
        let mut inflight_ticker_infos = FuturesUnordered::new();

        loop {
            // Get next order out of FuturesUnordered
            let next_open_response = if inflight_opens.is_empty() {
                Either::Left(std::future::pending())
            } else {
                Either::Right(inflight_opens.select_next_some())
            };

            // Get ticker info out of FuturesUnordered
            let next_ticker_info_response = if inflight_ticker_infos.is_empty() {
                Either::Left(std::future::pending())
            } else {
                Either::Right(inflight_ticker_infos.select_next_some())
            };

            tokio::select! {
                /*----- Handle Execution Requests from Traders ----- */
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

                            // Add ticker info like  precision, min quantity etc, to ExecutionManger
                            for instrument in request.instruments.iter() {
                                self.ticker_info.insert(instrument.clone(), TickerInfo::default());
                                inflight_ticker_infos.push(ExecutionRequestFuture::new(
                                    Exchange::PublicData::get_ticker_info(instrument.clone()),
                                    std::time::Duration::from_secs(5),
                                    request.clone()
                                ))
                            }

                        },
                        ExecutionRequest::Open(request) => {
                            inflight_opens.push(ExecutionRequestFuture::new(
                                    self.execution_client.open_order(request.clone()), //todo make input a clone
                                    self.request_timeout,
                                    request,
                                ));
                        }
                        ExecutionRequest::Cancel(_request) => {}
                        ExecutionRequest::CancelAll(_request) => {}
                        ExecutionRequest::Transfer(_request) => {}
                    }

                }


                /*----- Check Results of the FuturesUnordered ----- */
                open_response = next_open_response => {
                    println!("##### Open order #####");
                    println!("Open order res: {:#?}", open_response);

                }

                // Process ticker info
                ticker_info_response = next_ticker_info_response => {
                    // println!("##### Ticker Repsone #####");
                    // println!("{:#?}", self.ticker_info);

                    match ticker_info_response {
                        // If request is successful, loop over ticker_info hashmap and format the
                        // instrument to be exchange specific. If it matched the symbol from the
                        // result of the response, replace the value of the hashmap with this.
                        Ok(ticker_info) => {
                            let ticker = ticker_info.into();

                            let instrument = self.ticker_info.iter().find_map(|(key, _ )| {
                                let formatted_instrument = AssetFormatted::from((&Exchange::CLIENT, key));
                                match formatted_instrument.0 == ticker.symbol {
                                    true => Some(key.to_owned()),
                                    false => None
                                }
                            });

                            self.ticker_info.insert(instrument.unwrap(), ticker); // unwrap should not fail
                        },
                        // If unsuccessful, panic as this step is crusial
                        Err(error) => {
                            error!(
                                "ExecutionManager: {:#?}, failed to get ticker info with error message, {:#?}",
                                Exchange::CLIENT,
                                error
                            );
                            std::process::exit(1);
                        }
                    }
                }


                /*----- Process Execution Responses from Exchange ----- */
                Some(account_data) = self.account_data_rx.recv() => {
                    println!("##### Execution manger #####");
                    println!("Account Data: {:#?}", account_data);
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
