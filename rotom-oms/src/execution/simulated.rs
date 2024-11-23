use chrono::Utc;
use rotom_data::MarketMeta;

use crate::model::order::{Order, RequestOpen};

use super::{error::ExecutionError, Fees, FillEvent, FillGenerator};

/*----- */
// Config
/*----- */
pub struct Config {
    pub simulated_fees_pct: Fees,
}

/*----- */
// Simulated Execution
/*----- */
pub struct SimulatedExecution {
    fee_pct: Fees,
}

impl SimulatedExecution {
    pub fn new(config: Config) -> Self {
        Self {
            fee_pct: config.simulated_fees_pct,
        }
    }

    fn calculate_fill_value_gross(order: &Order<RequestOpen>) -> f64 {
        order.state.quantity.abs() * order.state.price
    }

    fn calculate_fees(&self, fill_value_gross: &f64) -> Fees {
        Fees {
            exchange: self.fee_pct.exchange * fill_value_gross,
            slippage: self.fee_pct.slippage * fill_value_gross,
            network: self.fee_pct.network * fill_value_gross,
        }
    }
}

/*----- */
// Impl the ExecutionClient
/*----- */
impl FillGenerator for SimulatedExecution {
    fn generate_fill(&self, order: &Order<RequestOpen>) -> Result<FillEvent, ExecutionError> {
        let fill_value_gross = SimulatedExecution::calculate_fill_value_gross(order);

        Ok(FillEvent {
            time: Utc::now(),
            exchange: order.exchange,
            instrument: order.instrument.clone(),
            market_meta: MarketMeta {
                close: order.state.price,
                time: Utc::now(),
            },
            decision: order.state.decision,
            quantity: order.state.quantity,
            fill_value_gross,
            fees: self.calculate_fees(&fill_value_gross),
        })
    }
}
