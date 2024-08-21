use chrono::Utc;

use crate::oms::OrderEvent;

use super::{error::ExecutionError, ExecutionClient, Fees, FillEvent};

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

    fn calculate_fill_value_gross(order: &OrderEvent) -> f64 {
        order.quantity.abs() * order.market_meta.close
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
impl ExecutionClient for SimulatedExecution {
    fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError> {
        let fill_value_gross = SimulatedExecution::calculate_fill_value_gross(order);
    
        Ok(FillEvent {
            time: Utc::now(),
            exchange: order.exchange,
            instrument: order.instrument.clone(),
            market_meta: order.market_meta,
            decision: order.decision,
            quantity: order.quantity,
            fill_value_gross,
            fees: self.calculate_fees(&fill_value_gross),
        })
    }
}