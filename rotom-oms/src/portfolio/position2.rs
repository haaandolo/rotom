use chrono::{DateTime, Utc};
use rotom_data::{
    model::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument},
    AssetFormatted, ExchangeAssetId,
};
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    execution::{FeeAmount, Fees, FillEvent},
    model::{
        account_response::OrderResponse,
        balance::Balance,
        order::{OrderEvent, OrderState},
        Side,
    },
};

use super::error::PortfolioError;

/*----- */
// Position
/*----- */
#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Position2 {
    // Time order was requested
    pub order_request_time: DateTime<Utc>,
    // Time order was last updated
    pub last_execution_time: DateTime<Utc>,
    // Unique identifier for the position. Exchange & asset combo e.g. BINANCESPOT_BTCUSDT
    pub position_id: ExchangeAssetId,
    // Exchange Id
    pub exchange: ExchangeId,
    // Asset formatted for give exchange. E.g. Binance = BTCUSDT & Poloniex = BTC_USDT
    pub asset: AssetFormatted,
    // Buy or Sell.
    pub side: Side,
    // Cumulative quantity
    pub quantity: f64,
    // Cumlative amount meaning the total fees the position as incurred. E.g filled with multiple limit orders this will represent the sum of all fee for the multiple orders
    pub fees: f64,
    // Symbol current close price
    pub current_symbol_price: f64,
    // abs(Quantity) * current_symbol_price
    pub current_value_gross: f64,
    // Filled gross amount of quote currency
    pub filled_gross: f64,
    // Enter average price excluding fees. Position.filled_gross / Position.quantity
    pub enter_avg_price: f64,
    // abs(Quantity) * enter_avg_price
    pub enter_value_gross: f64,

    //////////////////////////////////////////
    //////////////////////////////////////////

    // Unrealised P&L whilst the position is open
    pub unrealised_profit_loss: f64,
    // Realised P&L after the position has closed
    pub realised_profit_loss: f64,
}

impl Position2 {
    pub fn calculate_avg_price(&self) -> f64 {
        self.filled_gross / self.quantity
    }

    // Determine the [`Position`] entry [`Side`] by analysing the input [`FillEvent`].
    pub fn parse_entry_side(fill: &FillEvent) -> Result<Side, PortfolioError> {
        unimplemented!()
    }

    // Determines the [`Decision`] required to exit this [`Side`] (Buy or Sell) [`Position`].
    pub fn determine_exit_decision(&self) -> Decision {
        unimplemented!()
    }

    // Calculate the approximate [`Position::unrealised_profit_loss`] of a [`Position`].
    pub fn calculate_unrealised_profit_loss(&self) -> f64 {
        match self.side {
            Side::Buy => self.current_value_gross - self.enter_value_gross - self.fees,
            Side::Sell => self.enter_value_gross - self.current_value_gross - self.fees,
        }
    }

    // Calculate the exact [`Position::realised_profit_loss`] of a [`Position`].
    // pub fn calculate_realised_profit_loss(&self) -> f64 {
    //     match self.side {
    //         Side::Buy => self.
    //     }
    // }

    // Calculate the PnL return of a closed [`Position`] - assumed [`Position::realised_profit_loss`] is
    // appropriately calculated.
    pub fn calculate_profit_loss_return(&self) -> f64 {
        unimplemented!()
    }

    //////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////

    // PositionEnterer trait
    pub fn enter(_engine_id: Uuid, order: &OrderEvent) -> Position2 {
        Position2 {
            order_request_time: order.order_request_time,
            last_execution_time: order.last_execution_time.unwrap(),
            position_id: ExchangeAssetId::from((&order.exchange, &order.instrument)),
            exchange: order.exchange,
            asset: AssetFormatted::from((&order.exchange, &order.instrument)),
            side: Side::from(order.decision),
            quantity: order.cumulative_quantity,
            fees: order.fees,
            current_symbol_price: 0.0,
            current_value_gross: 0.0,
            filled_gross: 0.0,
            enter_avg_price: 0.0,
            unrealised_profit_loss: -order.fees,
            realised_profit_loss: 0.0,
            enter_value_gross: order.filled_gross,
        }
    }

    // PosiitionUpdater trait
    pub fn market_update(&mut self, market: &MarketEvent<DataKind>) -> Option<PositionUpdate2> {
        let close = match &market.event_data {
            DataKind::OrderBook(event_book) => event_book.weighted_midprice()?,
            DataKind::Trade(event_trade) => event_trade.trade.price,
        };

        self.current_symbol_price = close;
        self.current_value_gross = close * self.quantity.abs();
        self.unrealised_profit_loss = self.calculate_unrealised_profit_loss();

        Some(PositionUpdate2::from(self))
    }

    pub fn order_update(&mut self, order: &OrderEvent) {
        self.last_execution_time = order.last_execution_time.unwrap_or_default(); // todo?
        self.quantity = order.cumulative_quantity;
        self.filled_gross = order.filled_gross;
        self.fees = order.fees;
        self.enter_avg_price = self.calculate_avg_price();
        self.unrealised_profit_loss = self.calculate_unrealised_profit_loss();
    }

    // PositionExiter trait
    pub fn exit(
        &mut self,
        account_data: &OrderResponse,
        order: &OrderEvent,
    ) -> Result<PositionExit2, PortfolioError> {
        if order.is_order_complete() {
            return Err(PortfolioError::CannotExitPositionWithEntryFill);
        } else {
            unimplemented!()
        }
    }
}

/*----- */
// Position update
/*----- */
#[derive(Debug)]
pub struct PositionUpdate2 {
    pub position_id: String,
    pub update_time: DateTime<Utc>,
    pub current_symbol_price: f64,
    pub current_value_gross: f64,
    pub unrealised_profit_loss: f64,
}

impl From<&mut Position2> for PositionUpdate2 {
    fn from(updated_position: &mut Position2) -> Self {
        Self {
            position_id: updated_position.position_id.0.clone(),
            update_time: updated_position.last_execution_time,
            current_symbol_price: updated_position.current_symbol_price,
            current_value_gross: updated_position.current_value_gross,
            unrealised_profit_loss: updated_position.unrealised_profit_loss,
        }
    }
}

/*----- */
// Position Exit
/*----- */
#[derive(Debug)]
pub struct PositionExit2 {
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

impl TryFrom<&mut Position2> for PositionExit2 {
    type Error = PortfolioError;

    fn try_from(exited_position: &mut Position2) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
