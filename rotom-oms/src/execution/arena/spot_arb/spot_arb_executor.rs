use rotom_data::error::SocketError;
use std::fmt::Debug;

use crate::exchange::ExecutionClient;

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
}

impl<LiquidExchange, IlliquidExchange> SpotArbExecutor<LiquidExchange, IlliquidExchange>
where
    LiquidExchange: ExecutionClient + 'static,
    IlliquidExchange: ExecutionClient + 'static,
{
    pub async fn new() -> Result<SpotArbExecutor<LiquidExchange, IlliquidExchange>, SocketError> {
        let liquid_exchange_http = LiquidExchange::new()?;
        let illiquid_exchange_http = IlliquidExchange::new()?;

        Ok(SpotArbExecutor {
            liquid_exchange: liquid_exchange_http,
            illiquid_exchange: illiquid_exchange_http,
        })
    }
}

// /*----- */
// // Impl FillGenerator for SpotArbArena
// /*----- */
// impl<LiquidExchange, IlliquidExchange> FillGenerator
//     for SpotArbExecutor<LiquidExchange, IlliquidExchange>
// where
//     LiquidExchange: ExecutionClient,
//     IlliquidExchange: ExecutionClient,
// {
//     fn generate_fill(&mut self, order: OrderEvent) -> Result<FillEvent, ExecutionError> {
//         let asset = AssetFormatted::from((&order.exchange, &order.instrument)).0;
//         let existing_order = self.orders.get_mut(&asset);

//         // match existing_order {
//         //     Some(order) => {}
//         //     None => self.orders.insert(asset, order),
//         // }

//         Ok(FillEvent {
//             time: Utc::now(),
//             exchange: ExchangeId::BinanceSpot,
//             instrument: Instrument {
//                 base: "op".to_string(),
//                 quote: "usdt".to_string(),
//             },
//             market_meta: MarketMeta {
//                 time: Utc::now(),
//                 close: 0.0,
//             },
//             decision: rotom_strategy::Decision::Long,
//             quantity: 0.0,
//             fill_value_gross: 0.0,
//             fees: Fees {
//                 exchange: 0.0,
//                 network: 0.0,
//                 slippage: 0.0,
//             },
//         })
//     }
// }
