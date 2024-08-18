use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    execution::{FeeAmount, Fees, FillEvent},
    strategy::Decision,
};

use super::{error::PortfolioError, Balance};

/*----- */
// Position traits
/*----- */
pub trait PositionEnterer {
    fn enter(engine_id: Uuid, fill: &FillEvent) -> Result<Position, PortfolioError>;
}

pub trait PositionUpdater {
    fn update(&mut self, market: &MarketEvent<DataKind>) -> Option<PositionUpdate>;
}

pub trait PositionExiter {
    fn exit(&mut self, balance: Balance, fill: &FillEvent) -> Result<PositionExit, PortfolioError>;
}

/*----- */
// Position
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Position {
    // Unique identifier for a [`Position`] generated from an engine_id, [`Exchange`] & [`Instrument`].
    pub position_id: PositionId,

    // Metadata detailing trace UUIDs, timestamps & equity associated with entering, updating & exiting.
    pub meta: PositionMeta,

    // [`Exchange`] associated with this [`Position`].
    pub exchange: ExchangeId,

    // [`Instrument`] associated with this [`Position`].
    pub instrument: Instrument,

    // Buy or Sell.
    // Notes:
    // - Side::Buy considered synonymous with Long.
    // - Side::Sell considered synonymous with Short.
    pub side: Side,

    // +ve or -ve quantity of symbol contracts opened.
    pub quantity: f64,

    // All fees types incurred from entering a [`Position`], and their associated [`FeeAmount`].
    pub enter_fees: Fees,

    // Total of enter_fees incurred. Sum of every [`FeeAmount`] in [`Fees`] when entering a [`Position`].
    pub enter_fees_total: FeeAmount,

    // Enter average price excluding the entry_fees_total.
    pub enter_avg_price_gross: f64,

    // abs(Quantity) * enter_avg_price_gross.
    pub enter_value_gross: f64,

    // All fees types incurred from exiting a [`Position`], and their associated [`FeeAmount`].
    pub exit_fees: Fees,

    // Total of exit_fees incurred. Sum of every [`FeeAmount`] in [`Fees`] when entering a [`Position`].
    pub exit_fees_total: FeeAmount,

    // Exit average price excluding the exit_fees_total.
    pub exit_avg_price_gross: f64,

    // abs(Quantity) * exit_avg_price_gross.
    pub exit_value_gross: f64,

    // Symbol current close price.
    pub current_symbol_price: f64,

    // abs(Quantity) * current_symbol_price.
    pub current_value_gross: f64,

    // Unrealised P&L whilst the [`Position`] is open.
    pub unrealised_profit_loss: f64,

    // Realised P&L after the [`Position`] has closed.
    pub realised_profit_loss: f64,
}

impl Position {
    pub fn builder() -> PositionBuilder {
        PositionBuilder::new()
    }

    pub fn calculate_avg_price_gross(fill: &FillEvent) -> f64 {
        (fill.fill_value_gross / fill.quantity).abs()
    }

    // Determine the [`Position`] entry [`Side`] by analysing the input [`FillEvent`].
    pub fn parse_entry_side(fill: &FillEvent) -> Result<Side, PortfolioError> {
        match fill.decision {
            Decision::Long if fill.quantity.is_sign_positive() => Ok(Side::Buy),
            Decision::Short if fill.quantity.is_sign_negative() => Ok(Side::Sell),
            Decision::CloseLong | Decision::CloseShort => {
                Err(PortfolioError::CannotEnterPositionWithExitFill)
            }
            _ => Err(PortfolioError::ParseEntrySide),
        }
    }

    // Determines the [`Decision`] required to exit this [`Side`] (Buy or Sell) [`Position`].
    pub fn determine_exit_decision(&self) -> Decision {
        match self.side {
            Side::Buy => Decision::CloseLong,
            Side::Sell => Decision::CloseShort,
        }
    }

    // Calculate the approximate [`Position::unrealised_profit_loss`] of a [`Position`].
    pub fn calculate_unrealised_profit_loss(&self) -> f64 {
        let approx_total_fees = self.enter_fees_total * 2.0;

        match self.side {
            Side::Buy => self.current_value_gross - self.enter_value_gross - approx_total_fees,
            Side::Sell => self.enter_value_gross - self.current_value_gross - approx_total_fees,
        }
    }

    // Calculate the exact [`Position::realised_profit_loss`] of a [`Position`].
    pub fn calculate_realised_profit_loss(&self) -> f64 {
        let total_fees = self.enter_fees_total + self.exit_fees_total;

        match self.side {
            Side::Buy => self.exit_value_gross - self.enter_value_gross - total_fees,
            Side::Sell => self.enter_value_gross - self.exit_value_gross - total_fees,
        }
    }

    // Calculate the PnL return of a closed [`Position`] - assumed [`Position::realised_profit_loss`] is
    // appropriately calculated.
    pub fn calculate_profit_loss_return(&self) -> f64 {
        self.realised_profit_loss / self.enter_value_gross
    }
}

/*----- */
// Impl PostionEnterer for Position
/*----- */
impl PositionEnterer for Position {
    fn enter(engine_id: Uuid, fill: &FillEvent) -> Result<Position, PortfolioError> {
        let metadata = PositionMeta {
            enter_time: fill.market_meta.time,
            update_time: fill.time,
            exit_balance: None,
        };

        // Enter fees
        let enter_fees_total = fill.fees.calculate_total_fees();

        // Enter price
        let enter_avg_price_gross = Position::calculate_avg_price_gross(fill);

        // Unrealised profit & loss
        let unrealised_profit_loss = -enter_fees_total * 2.0;

        Ok(Position {
            position_id: determine_position_id(engine_id, &fill.exchange, &fill.instrument),
            exchange: fill.exchange,
            instrument: fill.instrument.clone(),
            meta: metadata,
            side: Position::parse_entry_side(fill)?,
            quantity: fill.quantity,
            enter_fees: fill.fees,
            enter_fees_total,
            enter_avg_price_gross,
            enter_value_gross: fill.fill_value_gross,
            exit_fees: Fees::default(),
            exit_fees_total: 0.0,
            exit_avg_price_gross: 0.0,
            exit_value_gross: 0.0,
            current_symbol_price: enter_avg_price_gross,
            current_value_gross: fill.fill_value_gross,
            unrealised_profit_loss,
            realised_profit_loss: 0.0,
        })
    }
}

/*----- */
// Impl PostionUpdater for Position
/*----- */
impl PositionUpdater for Position {
    fn update(&mut self, market: &MarketEvent<DataKind>) -> Option<PositionUpdate> {
        let close = match &market.event_data {
            DataKind::OrderBook(event_book) => event_book.bids[0].price, // TODO: change to volumme_weighted_mid_price
            DataKind::Trade(event_trade) => event_trade.trade.price,
        };

        self.meta.update_time = market.exchange_time;
        self.current_symbol_price = close;
        self.current_value_gross = close * self.quantity.abs();
        self.unrealised_profit_loss = self.calculate_unrealised_profit_loss();

        Some(PositionUpdate::from(self))
    }
}

/*----- */
// Position ID
/*----- */
impl PositionExiter for Position {
    fn exit(&mut self, mut balance: Balance, fill: &FillEvent) -> Result<PositionExit, PortfolioError> {
        if fill.decision.is_entry() {
            return Err(PortfolioError::CannotExitPositionWithEntryFill);
        }

        // Exit fees
        self.exit_fees = fill.fees;
        self.exit_fees_total = fill.fees.calculate_total_fees();

        // Exit value & price
        self.exit_value_gross = fill.fill_value_gross;
        self.exit_avg_price_gross = Position::calculate_avg_price_gross(fill);

        // Result profit & loss
        self.realised_profit_loss = self.calculate_realised_profit_loss();
        self.unrealised_profit_loss = self.realised_profit_loss;

        // Update meta
        balance.total += self.realised_profit_loss;
        self.meta.update_time = fill.time;
        self.meta.exit_balance = Some(balance);

        PositionExit::try_from(self)
    }
}

/*----- */
// Position ID
/*----- */
pub type PositionId = String;

pub fn determine_position_id(
    engine_id: Uuid,
    exchange: &ExchangeId,
    instrument: &Instrument,
) -> PositionId {
    format!("{}_{}_{}", engine_id, exchange, instrument)
}

/*----- */
// Position update
/*----- */
#[derive(Debug)]
pub struct PositionUpdate {
    pub position_id: String,
    pub update_time: DateTime<Utc>,
    pub current_symbol_price: f64,
    pub current_value_gross: f64,
    pub unrealised_profit_loss: f64,
}

impl From<&mut Position> for PositionUpdate {
    fn from(updated_position: &mut Position) -> Self {
        Self {
            position_id: updated_position.position_id.clone(),
            update_time: updated_position.meta.update_time,
            current_symbol_price: updated_position.current_symbol_price,
            current_value_gross: updated_position.current_value_gross,
            unrealised_profit_loss: updated_position.unrealised_profit_loss,
        }
    }
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
// Position Exit
/*----- */
#[derive(Debug)]
pub struct PositionExit {
    // Unique identifier for a [`Position`], generated from an exchange, symbol, and enter_time.
    pub position_id: String,

    // [`FillEvent`] timestamp that triggered the exiting of this [`Position`].
    pub exit_time: DateTime<Utc>,

    // Portfolio [`Balance`] calculated at the point of exiting a [`Position`].
    pub exit_balance: Balance,

    // All fees types incurred from exiting a [`Position`], and their associated [`FeeAmount`].
    pub exit_fees: Fees,

    // Total of exit_fees incurred. Sum of every [`FeeAmount`] in [`Fees`] when entering a [`Position`].
    pub exit_fees_total: FeeAmount,

    // Exit average price excluding the exit_fees_total.
    pub exit_avg_price_gross: f64,

    // abs(Quantity) * exit_avg_price_gross.
    pub exit_value_gross: f64,

    // Realised P&L after the [`Position`] has closed.
    pub realised_profit_loss: f64,
}

impl TryFrom<&mut Position> for PositionExit {
    type Error = PortfolioError;

    fn try_from(exited_position: &mut Position) -> Result<Self, Self::Error> {
        Ok(Self {
            position_id: exited_position.position_id.clone(),
            exit_time: exited_position.meta.update_time,
            exit_balance: exited_position.meta.exit_balance.ok_or(PortfolioError::PositionExit)?,
            exit_fees: exited_position.exit_fees,
            exit_fees_total: exited_position.exit_fees_total,
            exit_avg_price_gross: exited_position.exit_avg_price_gross,
            exit_value_gross: exited_position.exit_value_gross,
            realised_profit_loss: exited_position.realised_profit_loss,
        })
    }
}

/*----- */
// Position Meta
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct PositionMeta {
    // [`FillEvent`] timestamp that triggered the entering of this [`Position`].
    pub enter_time: DateTime<Utc>,

    // Timestamp of the last event to trigger a [`Position`] state change (enter, update, exit).
    pub update_time: DateTime<Utc>,

    // Portfolio [`Balance`] calculated at the point of exiting a [`Position`].
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

/*----- */
// Position Builder
/*----- */
#[derive(Debug, Default)]
pub struct PositionBuilder {
    pub position_id: Option<PositionId>,
    pub exchange: Option<ExchangeId>,
    pub instrument: Option<Instrument>,
    pub meta: Option<PositionMeta>,
    pub side: Option<Side>,
    pub quantity: Option<f64>,
    pub enter_fees: Option<Fees>,
    pub enter_fees_total: Option<FeeAmount>,
    pub enter_avg_price_gross: Option<f64>,
    pub enter_value_gross: Option<f64>,
    pub exit_fees: Option<Fees>,
    pub exit_fees_total: Option<FeeAmount>,
    pub exit_avg_price_gross: Option<f64>,
    pub exit_value_gross: Option<f64>,
    pub current_symbol_price: Option<f64>,
    pub current_value_gross: Option<f64>,
    pub unrealised_profit_loss: Option<f64>,
    pub realised_profit_loss: Option<f64>,
}

impl PositionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn position_id(self, value: PositionId) -> Self {
        Self {
            position_id: Some(value),
            ..self
        }
    }

    pub fn exchange(self, value: ExchangeId) -> Self {
        Self {
            exchange: Some(value),
            ..self
        }
    }

    pub fn instrument(self, value: Instrument) -> Self {
        Self {
            instrument: Some(value),
            ..self
        }
    }

    pub fn meta(self, value: PositionMeta) -> Self {
        Self {
            meta: Some(value),
            ..self
        }
    }

    pub fn side(self, value: Side) -> Self {
        Self {
            side: Some(value),
            ..self
        }
    }

    pub fn quantity(self, value: f64) -> Self {
        Self {
            quantity: Some(value),
            ..self
        }
    }

    pub fn enter_fees(self, value: Fees) -> Self {
        Self {
            enter_fees: Some(value),
            ..self
        }
    }

    pub fn enter_fees_total(self, value: FeeAmount) -> Self {
        Self {
            enter_fees_total: Some(value),
            ..self
        }
    }

    pub fn enter_avg_price_gross(self, value: f64) -> Self {
        Self {
            enter_avg_price_gross: Some(value),
            ..self
        }
    }

    pub fn enter_value_gross(self, value: f64) -> Self {
        Self {
            enter_value_gross: Some(value),
            ..self
        }
    }

    pub fn exit_fees(self, value: Fees) -> Self {
        Self {
            exit_fees: Some(value),
            ..self
        }
    }

    pub fn exit_fees_total(self, value: FeeAmount) -> Self {
        Self {
            exit_fees_total: Some(value),
            ..self
        }
    }

    pub fn exit_avg_price_gross(self, value: f64) -> Self {
        Self {
            exit_avg_price_gross: Some(value),
            ..self
        }
    }

    pub fn exit_value_gross(self, value: f64) -> Self {
        Self {
            exit_value_gross: Some(value),
            ..self
        }
    }

    pub fn current_symbol_price(self, value: f64) -> Self {
        Self {
            current_symbol_price: Some(value),
            ..self
        }
    }

    pub fn current_value_gross(self, value: f64) -> Self {
        Self {
            current_value_gross: Some(value),
            ..self
        }
    }

    pub fn unrealised_profit_loss(self, value: f64) -> Self {
        Self {
            unrealised_profit_loss: Some(value),
            ..self
        }
    }

    pub fn realised_profit_loss(self, value: f64) -> Self {
        Self {
            realised_profit_loss: Some(value),
            ..self
        }
    }

    pub fn build(self) -> Result<Position, PortfolioError> {
        Ok(Position {
            position_id: self
                .position_id
                .ok_or(PortfolioError::BuilderIncomplete("position_id"))?,
            exchange: self
                .exchange
                .ok_or(PortfolioError::BuilderIncomplete("exchange"))?,
            instrument: self
                .instrument
                .ok_or(PortfolioError::BuilderIncomplete("instrument"))?,
            meta: self.meta.ok_or(PortfolioError::BuilderIncomplete("meta"))?,
            side: self.side.ok_or(PortfolioError::BuilderIncomplete("side"))?,
            quantity: self
                .quantity
                .ok_or(PortfolioError::BuilderIncomplete("quantity"))?,
            enter_fees: self
                .enter_fees
                .ok_or(PortfolioError::BuilderIncomplete("enter_fees"))?,
            enter_fees_total: self
                .enter_fees_total
                .ok_or(PortfolioError::BuilderIncomplete("enter_fees_total"))?,
            enter_avg_price_gross: self
                .enter_avg_price_gross
                .ok_or(PortfolioError::BuilderIncomplete("enter_avg_price_gross"))?,
            enter_value_gross: self
                .enter_value_gross
                .ok_or(PortfolioError::BuilderIncomplete("enter_value_gross"))?,
            exit_fees: self
                .exit_fees
                .ok_or(PortfolioError::BuilderIncomplete("exit_fees"))?,
            exit_fees_total: self
                .exit_fees_total
                .ok_or(PortfolioError::BuilderIncomplete("exit_fees_total"))?,
            exit_avg_price_gross: self
                .exit_avg_price_gross
                .ok_or(PortfolioError::BuilderIncomplete("exit_avg_price_gross"))?,
            exit_value_gross: self
                .exit_value_gross
                .ok_or(PortfolioError::BuilderIncomplete("exit_value_gross"))?,
            current_symbol_price: self
                .current_symbol_price
                .ok_or(PortfolioError::BuilderIncomplete("current_symbol_price"))?,
            current_value_gross: self
                .current_value_gross
                .ok_or(PortfolioError::BuilderIncomplete("current_value_gross"))?,
            unrealised_profit_loss: self
                .unrealised_profit_loss
                .ok_or(PortfolioError::BuilderIncomplete("unrealised_profit_loss"))?,
            realised_profit_loss: self
                .realised_profit_loss
                .ok_or(PortfolioError::BuilderIncomplete("realised_profit_loss"))?,
        })
    }
}
