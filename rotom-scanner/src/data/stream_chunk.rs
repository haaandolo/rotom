use std::fmt::Debug;

use rotom_data::{
    exchange::{PublicHttpConnector, PublicStreamConnector},
    shared::subscription_models::{ExchangeId, StreamKind},
};

#[derive(Debug, Default)]
pub struct StreamChunks(pub Vec<Vec<(ExchangeId, String, String, StreamKind)>>);

impl StreamChunks {
    pub async fn add_exchange<Exchange>(mut self) -> Self
    where
        Exchange: PublicHttpConnector + PublicStreamConnector + 'static,
    {
        let tickers = Exchange::get_usdt_pair().await.unwrap(); // Unwrap allowed as we want this to fail if unsuccessful
        let exchange_id = <Exchange as PublicHttpConnector>::ID;
        let ws_chunk_size = Exchange::ws_chunk_size();
        let orderbook_type = Exchange::ORDERBOOK;
        let trade_type = Exchange::TRADE;

        for chunk in tickers.chunks(ws_chunk_size) {
            let mut orderbook_chunk = Vec::new();
            let mut trade_chunk = Vec::new();

            for (base, quote) in chunk.iter() {
                orderbook_chunk.push((exchange_id, base.clone(), quote.clone(), orderbook_type));

                trade_chunk.push((exchange_id, base.clone(), quote.clone(), trade_type));
            }

            self.0.push(orderbook_chunk);
            self.0.push(trade_chunk);
        }

        self
    }

    pub fn build(self) -> Vec<Vec<(ExchangeId, String, String, StreamKind)>> {
        self.0
    }
}
