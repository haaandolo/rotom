use chrono::Utc;
use rotom_data::{
    error::SocketError,
    shared::subscription_models::{ExchangeId, Instrument},
    MarketMeta,
};
use std::{collections::HashMap, fmt::Debug};
use tokio::sync::mpsc;

use crate::{
    exchange::ExecutionClient,
    execution::{error::ExecutionError, Fees, FillEvent, FillGenerator},
    model::{
        account_data::AccountData,
        order::{AssetFormatted, OrderEvent},
    },
};

/*----- */
// Spot Arb Executor - combines exchange execution client's for spot arb
/*----- */
#[derive(Debug)]
pub struct SpotArbExecutor<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
{
    pub liquid_exchange: LiquidExchange,
    pub illiquid_exchange: IlliquidExchange,
    pub streams: mpsc::UnboundedReceiver<AccountData>,
    pub orders: HashMap<String, OrderEvent>,
}

impl<LiquidExchange, IlliquidExchange> SpotArbExecutor<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient + 'static,
    IlliquidExchange: ExecutionClient + 'static,
{
    pub async fn init() -> Result<SpotArbExecutor<LiquidExchange, IlliquidExchange>, SocketError> {
        unimplemented!()
        // // Convert first exchange ws to channel
        // let (liquid_exchange_tx, mut liquid_exchange_rx) = mpsc::unbounded_channel();
        // tokio::spawn(consume_account_data_ws::<LiquidExchange>(
        //     liquid_exchange_tx,
        // ));

        // // Convert second exchange ws to channel
        // let (illiquid_exchange_tx, mut illiquid_exchange_rx) = mpsc::unbounded_channel();
        // tokio::spawn(consume_account_data_ws::<IlliquidExchange>(
        //     illiquid_exchange_tx,
        // ));

        // // Combine channels into one
        // let (combined_tx, combined_rx) = mpsc::unbounded_channel();
        // let combined_tx_cloned = combined_tx.clone();
        // tokio::spawn(async move {
        //     while let Some(message) = liquid_exchange_rx.recv().await {
        //         let _ = combined_tx_cloned.send(message);
        //     }
        // });

        // tokio::spawn(async move {
        //     while let Some(message) = illiquid_exchange_rx.recv().await {
        //         let _ = combined_tx.send(message);
        //     }
        // });

        // // Init exchange http clients
        // let liquid_exchange_http = LiquidExchange::create_http_client()?;
        // let illiquid_exchange_http = IlliquidExchange::create_http_client()?;

        // Ok(SpotArbExecutor {
        //     liquid_exchange: liquid_exchange_http,
        //     illiquid_exchange: illiquid_exchange_http,
        //     streams: combined_rx,
        //     orders: HashMap::new(),
        // })
    }
}

/*----- */
// Impl FillGenerator for SpotArbArena
/*----- */
impl<LiquidExchange, IlliquidExchange> FillGenerator
    for SpotArbExecutor<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient,
    IlliquidExchange: ExecutionClient,
{
    fn generate_fill(&mut self, order: OrderEvent) -> Result<FillEvent, ExecutionError> {
        let asset = AssetFormatted::from((&order.exchange, &order.instrument)).0;
        let existing_order = self.orders.get_mut(&asset);

        // match existing_order {
        //     Some(order) => {}
        //     None => self.orders.insert(asset, order),
        // }

        Ok(FillEvent {
            time: Utc::now(),
            exchange: ExchangeId::BinanceSpot,
            instrument: Instrument {
                base: "op".to_string(),
                quote: "usdt".to_string(),
            },
            market_meta: MarketMeta {
                time: Utc::now(),
                close: 0.0,
            },
            decision: rotom_strategy::Decision::Long,
            quantity: 0.0,
            fill_value_gross: 0.0,
            fees: Fees {
                exchange: 0.0,
                network: 0.0,
                slippage: 0.0,
            },
        })
    }
}
