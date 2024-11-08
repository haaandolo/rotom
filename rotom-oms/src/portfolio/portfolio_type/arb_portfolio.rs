use rotom_data::{
    protocols::http::{client::RestClient, http_parser::StandardHttpParser},
    shared::subscription_models::ExchangeId,
};

use crate::exchange::{
    binance::{
        binance_client::{BinanceExecution, BINANCE_BASE_URL},
        request_builder::BinanceRequestBuilder,
    },
    ExecutionClient2,
};

#[derive(Debug)]
pub struct ArbPortfolio {
    pub exchanges: Vec<ExchangeId>,
}

impl ArbPortfolio {
    pub async fn init(&self) {
        for exchange in self.exchanges.iter() {
            match exchange {
                ExchangeId::BinanceSpot => {
                    let http_client = RestClient::new(
                        BINANCE_BASE_URL,
                        StandardHttpParser,
                        BinanceRequestBuilder,
                    );
                }
                ExchangeId::PoloniexSpot => (),
            }
        }
    }
}
