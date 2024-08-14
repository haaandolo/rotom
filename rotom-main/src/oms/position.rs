use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use rotom_data::shared::subscription_models::{ExchangeId, Instrument};
use serde::{Deserialize, Serialize};

use crate::execution::{FeeAmount, Fees};

use super::Balance;

/*----- */
// Position
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct Position {
    /// Unique identifier for a [`Position`] generated from an engine_id, [`Exchange`] & [`Instrument`].
    pub position_id: PositionId,

    /// Metadata detailing trace UUIDs, timestamps & equity associated with entering, updating & exiting.
    pub meta: PositionMeta,

    /// [`Exchange`] associated with this [`Position`].
    pub exchange: ExchangeId,

    /// [`Instrument`] associated with this [`Position`].
    pub instrument: Instrument,

    /// Buy or Sell.
    ///
    /// Notes:
    /// - Side::Buy considered synonymous with Long.
    /// - Side::Sell considered synonymous with Short.
    pub side: Side,

    /// +ve or -ve quantity of symbol contracts opened.
    pub quantity: f64,

    /// All fees types incurred from entering a [`Position`], and their associated [`FeeAmount`].
    pub enter_fees: Fees,

    /// Total of enter_fees incurred. Sum of every [`FeeAmount`] in [`Fees`] when entering a [`Position`].
    pub enter_fees_total: FeeAmount,

    /// Enter average price excluding the entry_fees_total.
    pub enter_avg_price_gross: f64,

    /// abs(Quantity) * enter_avg_price_gross.
    pub enter_value_gross: f64,

    /// All fees types incurred from exiting a [`Position`], and their associated [`FeeAmount`].
    pub exit_fees: Fees,

    /// Total of exit_fees incurred. Sum of every [`FeeAmount`] in [`Fees`] when entering a [`Position`].
    pub exit_fees_total: FeeAmount,

    /// Exit average price excluding the exit_fees_total.
    pub exit_avg_price_gross: f64,

    /// abs(Quantity) * exit_avg_price_gross.
    pub exit_value_gross: f64,

    /// Symbol current close price.
    pub current_symbol_price: f64,

    /// abs(Quantity) * current_symbol_price.
    pub current_value_gross: f64,

    /// Unrealised P&L whilst the [`Position`] is open.
    pub unrealised_profit_loss: f64,

    /// Realised P&L after the [`Position`] has closed.
    pub realised_profit_loss: f64,
}

/*----- */
// Position ID
/*----- */
pub type PositionId = String;

pub fn determine_position_id(
    // engine_id: Uuid,
    exchange: &ExchangeId,
    instrument: &Instrument,
) -> PositionId {
    format!("{}_{}", exchange, instrument)
}

/*----- */
// Position update
/*----- */
pub struct PositionUpdate {
    pub position_id: String,
    pub update_time: DateTime<Utc>,
    pub current_symbol_price: f64,
    pub current_value_gross: f64,
    pub unrealised_profit_loss: f64,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum Side {
    #[serde(alias = "buy", alias = "BUY", alias = "b")]
    Buy,
    #[serde(alias = "sell", alias = "SELL", alias = "s")]
    Sell,
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Side::Buy => "buy",
                Side::Sell => "sell",
            }
        )
    }
}

/*----- */
// Position Meta
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct PositionMeta {
    /// [`FillEvent`] timestamp that triggered the entering of this [`Position`].
    pub enter_time: DateTime<Utc>,

    /// Timestamp of the last event to trigger a [`Position`] state change (enter, update, exit).
    pub update_time: DateTime<Utc>,

    /// Portfolio [`Balance`] calculated at the point of exiting a [`Position`].
    pub exit_balance: Option<Balance>,
}

impl Default for PositionMeta {
    fn default() -> Self {
        Self {
            enter_time: Utc::now(),
            update_time: Utc::now(),
            exit_balance: None,
        }
    }
}
