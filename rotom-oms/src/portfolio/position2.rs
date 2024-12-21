use chrono::{DateTime, Utc};
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::{ExchangeId, Instrument},
    AssetFormatted, ExchangeAssetId,
};
use rotom_strategy::Decision;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    execution::{FeeAmount, Fees, FillEvent},
    model::{account_data::AccountDataOrder, balance::Balance, order::OrderEvent, Side},
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
    // +ve or -ve quantity of symbol contracts opened.
    pub quantity: f64,
    // Cumlative amount meaning the total fees the position as incurred. E.g filled with multiple limit orders this will represent the sum of all fee for the multiple orders
    pub fees: f64,
    // Symbol current close price
    pub current_symbol_price: f64,
    // abs(Quantity) * current_symbol_price
    pub current_value_gross: f64,

    //////////////////////////////////////////
    //////////////////////////////////////////

    // Unrealised P&L whilst the position is open
    pub unrealised_profit_loss: f64,
    // Realised P&L after the position has closed
    pub realised_profit_loss: f64,
}

impl Position2 {
    pub fn calculate_avg_price_gross(order_update: &AccountDataOrder) -> f64 {
        unimplemented!()
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
        unimplemented!()
    }

    // Calculate the exact [`Position::realised_profit_loss`] of a [`Position`].
    pub fn calculate_realised_profit_loss(&self) -> f64 {
        unimplemented!()
    }

    // Calculate the PnL return of a closed [`Position`] - assumed [`Position::realised_profit_loss`] is
    // appropriately calculated.
    pub fn calculate_profit_loss_return(&self) -> f64 {
        unimplemented!()
    }

    //////////////////////////////////////////////////////////////////
    //////////////////////////////////////////////////////////////////

    // PositionEnterer trait
    pub fn enter(
        engine_id: Uuid,
        account_data: &AccountDataOrder,
        order: &OrderEvent,
    ) -> Position2 {
        Position2 {
            order_request_time: order.order_request_time,
            last_execution_time: account_data.execution_time,
            position_id: ExchangeAssetId::from((&order.exchange, &order.instrument)),
            exchange: order.exchange,
            asset: AssetFormatted(account_data.asset.clone()),
            side: account_data.side,
            quantity: order.quantity,
            fees: account_data.fee,
            current_symbol_price: 0.0,
            current_value_gross: 0.0,
            unrealised_profit_loss: 0.0,
            realised_profit_loss: 0.0,
        }
    }

    // PosiitionUpdater trait
    pub fn update(&mut self, market: &MarketEvent<DataKind>) -> Option<PositionUpdate2> {
        let close = match &market.event_data {
            DataKind::OrderBook(event_book) => event_book.weighted_midprice()?,
            DataKind::Trade(event_trade) => event_trade.trade.price,
        };

        self.current_symbol_price = close;
        self.current_value_gross = close * self.quantity.abs();
        self.unrealised_profit_loss = self.calculate_unrealised_profit_loss();

        Some(PositionUpdate2::from(self))
    }

    // PositionExiter trait
    pub fn exit(
        &mut self,
        mut balance: Balance,
        fill: &FillEvent,
    ) -> Result<PositionExit2, PortfolioError> {
        unimplemented!()
    }

    pub fn exit_spot(
        &mut self,
        base_asset_balance: &mut Balance,
        quote_asset_balance: &mut Balance,
        fill: &FillEvent,
    ) -> Result<PositionExit2, PortfolioError> {
        unimplemented!()
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
        unimplemented!()
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
