use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::{
    exchange::{consume_account_data_stream, ExecutionClient},
    model::{account_data::AccountData, order::ExecutionRequest},
};

use super::builder::TraderId;



#[derive(Debug)]
pub struct TraderMetaData {
    pub update_tx: mpsc::Sender<AccountData>,
    pub orders: Vec<AccountData>,
}

impl TraderMetaData {
    pub fn new(update_tx: mpsc::Sender<AccountData>) -> Self {
        Self {
            update_tx,
            orders: Vec::with_capacity(5),
        }
    }
}

#[derive(Debug)]
pub struct ExecutionManager<Exchange>
where
    Exchange: ExecutionClient,
{
    pub execution_client: Exchange,
    pub execution_request_tx: mpsc::UnboundedSender<ExecutionRequest>,
    pub traders: HashMap<TraderId, TraderMetaData>,
    pub request_timeout: std::time::Duration,
}

impl<Exchange> ExecutionManager<Exchange>
where
    Exchange: ExecutionClient + 'static,
{
    pub fn init(trader_update_txs: HashMap<TraderId, mpsc::Sender<AccountData>>) -> Self {
        // Init channels
        let (account_data_tx, account_data_rx) = mpsc::unbounded_channel();
        let (execution_request_tx, execution_rx) = mpsc::unbounded_channel();

        // Init trader Hashmap
        let mut traders = HashMap::with_capacity(trader_update_txs.len());
        for (trader_id, update_tx) in trader_update_txs {
            traders.insert(trader_id, TraderMetaData::new(update_tx));
        }

        tokio::spawn(consume_account_data_stream::<Exchange>(account_data_tx));

        // futures::stream::select_all()

        Self {
            execution_client: Exchange::new(),
            execution_request_tx,
            traders,
            request_timeout: std::time::Duration::from_millis(100), // todo: make exchange specific and include in exeution client
        }
    }
}
