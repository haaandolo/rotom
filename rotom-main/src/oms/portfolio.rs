use std::collections::HashMap;

use chrono::Utc;
use rotom_data::event_models::market_event::{DataKind, MarketEvent};
use tracing::info;
use uuid::Uuid;

use crate::{
    data::{Market, MarketMeta},
    event::Event,
    execution::FillEvent,
    strategy::{Decision, Signal, SignalForceExit, SignalStrength},
};

use super::{
    allocator::OrderAllocator,
    error::PortfolioError,
    position::{
        determine_position_id, Position, PositionEnterer, PositionExiter, PositionUpdate,
        PositionUpdater, Side,
    },
    repository::{BalanceHandler, PositionHandler},
    risk::OrderEvaluator,
    Balance, FillUpdater, MarketUpdater, OrderEvent, OrderGenerator, OrderType,
};

/*----- */
// Porfolio lego
/*----- */
pub struct PortfolioLego<Repository, Allocator, RiskManager>
where
    Repository: PositionHandler + BalanceHandler,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
{
    pub engine_id: Uuid,
    pub markets: Vec<Market>,
    pub repository: Repository,
    pub allocator: Allocator,
    pub starting_cash: f64,
    pub risk: RiskManager,
}

/*----- */
// Metaportfolio
/*----- */
pub struct MetaPortfolio<Repository, Allocator, RiskManager>
where
    Repository: PositionHandler + BalanceHandler,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
{
    pub engine_id: Uuid,
    pub repository: Repository,
    pub allocation_manager: Allocator,
    pub risk_manager: RiskManager,
}

impl<Repository, Allocator, RiskManager> MetaPortfolio<Repository, Allocator, RiskManager>
where
    Repository: PositionHandler + BalanceHandler,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
{
    pub fn init(
        lego: PortfolioLego<Repository, Allocator, RiskManager>,
    ) -> Result<Self, PortfolioError> {
        let mut porfolio = Self {
            engine_id: lego.engine_id,
            repository: lego.repository,
            allocation_manager: lego.allocator,
            risk_manager: lego.risk,
        };

        porfolio.bootstrap_repository(lego.starting_cash, &lego.markets)?;

        Ok(porfolio)
    }

    pub fn bootstrap_repository(
        &mut self,
        starting_cash: f64,
        _markets: &Vec<Market>,
    ) -> Result<(), PortfolioError> {
        self.repository.set_balance(
            self.engine_id,
            Balance {
                time: Utc::now(),
                total: starting_cash,
                available: starting_cash,
            },
        )?;
        Ok(())
    }

    pub fn builder() -> MetaPortfolioBuilder<Repository, Allocator, RiskManager> {
        MetaPortfolioBuilder::new()
    }

    pub fn no_cash_to_enter_new_position(&mut self) -> Result<bool, PortfolioError> {
        self.repository
            .get_balance(self.engine_id)
            .map(|balance| balance.available == 0.0)
            .map_err(PortfolioError::RepositoryInteraction)
    }
}

/*----- */
// MetaPortfolio Builder
/*----- */
#[derive(Debug, Default)]
pub struct MetaPortfolioBuilder<Repository, Allocator, RiskManager>
where
    Repository: PositionHandler + BalanceHandler,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
{
    engine_id: Option<Uuid>,
    markets: Option<Vec<Market>>,
    starting_cash: Option<f64>,
    repository: Option<Repository>,
    allocation_manager: Option<Allocator>,
    risk_manager: Option<RiskManager>,
}

impl<Repository, Allocator, RiskManager> MetaPortfolioBuilder<Repository, Allocator, RiskManager>
where
    Repository: PositionHandler + BalanceHandler,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
{
    pub fn new() -> Self {
        Self {
            engine_id: None,
            markets: None,
            starting_cash: None,
            repository: None,
            allocation_manager: None,
            risk_manager: None,
        }
    }

    pub fn engine_id(self, value: Uuid) -> Self {
        Self {
            engine_id: Some(value),
            ..self
        }
    }

    pub fn markets(self, value: Vec<Market>) -> Self {
        Self {
            markets: Some(value),
            ..self
        }
    }

    pub fn starting_cash(self, value: f64) -> Self {
        Self {
            starting_cash: Some(value),
            ..self
        }
    }

    pub fn repository(self, value: Repository) -> Self {
        Self {
            repository: Some(value),
            ..self
        }
    }

    pub fn allocation_manager(self, value: Allocator) -> Self {
        Self {
            allocation_manager: Some(value),
            ..self
        }
    }

    pub fn risk_manager(self, value: RiskManager) -> Self {
        Self {
            risk_manager: Some(value),
            ..self
        }
    }

    pub fn build_init(
        self,
    ) -> Result<MetaPortfolio<Repository, Allocator, RiskManager>, PortfolioError> {
        let mut portfolio = MetaPortfolio {
            engine_id: self
                .engine_id
                .ok_or(PortfolioError::BuilderIncomplete("engine_id"))?,
            repository: self
                .repository
                .ok_or(PortfolioError::BuilderIncomplete("repository"))?,
            allocation_manager: self
                .allocation_manager
                .ok_or(PortfolioError::BuilderIncomplete("allocation_manager"))?,
            risk_manager: self
                .risk_manager
                .ok_or(PortfolioError::BuilderIncomplete("risk_manager"))?,
        };

        // Persist initial state in the Repository
        // TODO: stats and market
        portfolio.bootstrap_repository(
            self.starting_cash
                .ok_or(PortfolioError::BuilderIncomplete("starting_cash"))?,
            &self
                .markets
                .ok_or(PortfolioError::BuilderIncomplete("markets"))?,
            // self.statistic_config
            //     .ok_or(PortfolioError::BuilderIncomplete("statistic_config"))?,
        )?;

        Ok(portfolio)
    }
}

/*----- */
// Impl MarketUpdater for MetaPortfolio
/*----- */
impl<Repository, Allocator, RiskManager> MarketUpdater
    for MetaPortfolio<Repository, Allocator, RiskManager>
where
    Repository: PositionHandler + BalanceHandler,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
{
    fn update_from_market(
        &mut self,
        market: &MarketEvent<DataKind>,
    ) -> Result<Option<PositionUpdate>, PortfolioError> {
        let position_id =
            determine_position_id(self.engine_id, &market.exchange, &market.instrument);

        if let Some(mut position) = self.repository.get_open_position(&position_id)? {
            if let Some(position_update) = position.update(market) {
                self.repository.set_open_position(position)?;
                return Ok(Some(position_update));
            }
        }

        Ok(None)
    }
}

/*----- */
// Impl OrderGenerator for MetaPortfolio
/*----- */
impl<Repository, Allocator, RiskManager> OrderGenerator
    for MetaPortfolio<Repository, Allocator, RiskManager>
where
    Repository: PositionHandler + BalanceHandler,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
{
    fn generate_order(&mut self, signal: &Signal) -> Result<Option<OrderEvent>, PortfolioError> {
        let position_id =
            determine_position_id(self.engine_id, &signal.exchange, &signal.instrument);
        let position = self.repository.get_open_position(&position_id)?;

        if position.is_none() && self.no_cash_to_enter_new_position()? {
            return Ok(None);
        }

        let position = position.as_ref();
        let (signal_decision, signal_strength) =
            match parse_signal_decision(&position, &signal.signals) {
                None => return Ok(None),
                Some(net_signal) => net_signal,
            };

        let mut order = OrderEvent {
            time: Utc::now(),
            exchange: signal.exchange,
            instrument: signal.instrument.clone(),
            market_meta: signal.market_meta,
            decision: *signal_decision,
            quantity: 0.0,
            order_type: OrderType::default(),
        };

        self.allocation_manager
            .allocate_order(&mut order, position, *signal_strength);

        Ok(self.risk_manager.evaluate_order(order))
    }

    fn generate_exit_order(
        &mut self,
        signal: SignalForceExit,
    ) -> Result<Option<OrderEvent>, PortfolioError> {
        let position_id =
            determine_position_id(self.engine_id, &signal.exchange, &signal.instrument);

        let position = match self.repository.get_open_position(&position_id)? {
            None => {
                info!(
                    position_id = &*position_id,
                    outcome = "no forced exit OrderEvent generated",
                    "cannot generate forced exit OrderEvent for a Position that isn't open"
                );
                return Ok(None);
            }
            Some(position) => position,
        };

        Ok(Some(OrderEvent {
            time: Utc::now(),
            exchange: signal.exchange,
            instrument: signal.instrument,
            market_meta: MarketMeta {
                close: position.current_symbol_price,
                time: position.meta.update_time,
            },
            decision: position.determine_exit_decision(),
            quantity: 0.0 - position.quantity,
            order_type: OrderType::Market,
        }))
    }
}

/*----- */
// Impl FillUpdater for MetaPortfolio
/*----- */
impl<Repository, Allocator, RiskManager> FillUpdater
    for MetaPortfolio<Repository, Allocator, RiskManager>
where
    Repository: PositionHandler + BalanceHandler,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
{
    fn update_from_fill(&mut self, fill: &FillEvent) -> Result<Vec<Event>, PortfolioError> {
        let mut generate_events = Vec::with_capacity(2);
        let mut balance = self.repository.get_balance(self.engine_id)?;
        balance.time = fill.time;

        let position_id = determine_position_id(self.engine_id, &fill.exchange, &fill.instrument);

        match self.repository.remove_positions(&position_id)? {
            Some(mut position) => {
                let position_exit = position.exit(balance, fill)?;
                generate_events.push(Event::PositionExit(position_exit));

                balance.available += position.enter_value_gross
                    + position.realised_profit_loss
                    + position.enter_fees_total;
                balance.total += position.realised_profit_loss;

                // TODO: stats
                self.repository
                    .set_exited_position(self.engine_id, position)?;
            }

            None => {
                let position = Position::enter(self.engine_id, fill)?;
                generate_events.push(Event::PositionNew(position.clone()));

                balance.available += -position.enter_value_gross - position.enter_fees_total;
                self.repository.set_open_position(position)?;
            }
        }

        generate_events.push(Event::Balance(balance));
        self.repository.set_balance(self.engine_id, balance)?;

        Ok(generate_events)
    }
}

/*----- */
// Parse Signal Decision
/*----- */
pub fn parse_signal_decision<'a>(
    position: &'a Option<&Position>,
    signals: &'a HashMap<Decision, SignalStrength>,
) -> Option<(&'a Decision, &'a SignalStrength)> {
    // Determine the presence of signals in the provided signals HashMap
    let signal_close_long = signals.get_key_value(&Decision::CloseLong);
    let signal_long = signals.get_key_value(&Decision::Long);
    let signal_close_short = signals.get_key_value(&Decision::CloseShort);
    let signal_short = signals.get_key_value(&Decision::Short);

    // If an existing Position exists, check for net close signals
    if let Some(position) = position {
        return match position.side {
            Side::Buy if signal_close_long.is_some() => signal_close_long,
            Side::Sell if signal_close_short.is_some() => signal_close_short,
            _ => None,
        };
    }

    // Else check for net open signals
    match (signal_long, signal_short) {
        (Some(signal_long), None) => Some(signal_long),
        (None, Some(signal_short)) => Some(signal_short),
        _ => None,
    }
}
