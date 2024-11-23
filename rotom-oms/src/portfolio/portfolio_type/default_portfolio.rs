use std::{collections::HashMap, marker::PhantomData};

use chrono::Utc;
use rotom_data::{
    event_models::market_event::{DataKind, MarketEvent},
    Market, MarketId, MarketMeta,
};
use rotom_strategy::{Decision, Signal, SignalForceExit, SignalStrength};
use tracing::info;
use uuid::Uuid;

use crate::{
    event::Event,
    execution::FillEvent,
    model::{
        balance::Balance, order::{Order, RequestOpen}, ClientOrderId, OrderKind, Side
    },
    portfolio::{
        allocator::OrderAllocator,
        error::PortfolioError,
        persistence::{error::RepositoryError, BalanceHandler, PositionHandler, StatisticHandler},
        position::{
            determine_position_id, Position, PositionEnterer, PositionExiter, PositionId,
            PositionUpdate, PositionUpdater,
        },
        risk_manager::OrderEvaluator,
        OrderEvent, OrderType,
    },
    statistic::summary::{Initialiser, PositionSummariser},
};

use super::{FillUpdater, MarketUpdater, OrderGenerator};

/*----- */
// Porfolio lego
/*----- */
pub struct PortfolioLego<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
{
    pub engine_id: Uuid,
    pub markets: Vec<Market>,
    pub repository: Repository,
    pub allocator: Allocator,
    pub starting_cash: f64,
    pub risk: RiskManager,
    pub statistic_config: Statistic::Config,
}

/*----- */
// Metaportfolio
/*----- */
#[derive(Debug)]
pub struct MetaPortfolio<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
{
    pub engine_id: Uuid,
    pub repository: Repository,
    pub allocation_manager: Allocator,
    pub risk_manager: RiskManager,
    pub _statistic_marker: PhantomData<Statistic>,
}

impl<Repository, Allocator, RiskManager, Statistic>
    MetaPortfolio<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
{
    pub fn init(
        lego: PortfolioLego<Repository, Allocator, RiskManager, Statistic>,
    ) -> Result<Self, PortfolioError> {
        let mut porfolio = Self {
            engine_id: lego.engine_id,
            repository: lego.repository,
            allocation_manager: lego.allocator,
            risk_manager: lego.risk,
            _statistic_marker: PhantomData,
        };

        porfolio.bootstrap_repository(lego.starting_cash, &lego.markets, lego.statistic_config)?;

        Ok(porfolio)
    }

    pub fn bootstrap_repository(
        &mut self,
        starting_cash: f64,
        markets: &[Market],
        statistic_config: Statistic::Config,
    ) -> Result<(), PortfolioError> {
        // Persist initial Balance (total & available)
        self.repository.set_balance(
            self.engine_id,
            Balance {
                total: starting_cash,
                available: starting_cash,
            },
        )?;

        // Persist initial MetaPortfolio Statistics for every Market
        markets.iter().try_for_each(|market| {
            self.repository
                .set_statistics(MarketId::from(market), Statistic::init(statistic_config))
                .map_err(PortfolioError::RepositoryInteraction)
        })
    }

    pub fn builder() -> MetaPortfolioBuilder<Repository, Allocator, RiskManager, Statistic> {
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
pub struct MetaPortfolioBuilder<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
{
    engine_id: Option<Uuid>,
    markets: Option<Vec<Market>>,
    starting_cash: Option<f64>,
    repository: Option<Repository>,
    allocation_manager: Option<Allocator>,
    risk_manager: Option<RiskManager>,
    statistic_config: Option<Statistic::Config>,
    _statistic_marker: Option<PhantomData<Statistic>>,
}

impl<Repository, Allocator, RiskManager, Statistic>
    MetaPortfolioBuilder<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
{
    pub fn new() -> Self {
        Self {
            engine_id: None,
            markets: None,
            starting_cash: None,
            repository: None,
            allocation_manager: None,
            risk_manager: None,
            statistic_config: None,
            _statistic_marker: None,
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

    pub fn statistic_config(self, value: Statistic::Config) -> Self {
        Self {
            statistic_config: Some(value),
            ..self
        }
    }

    pub fn build_init(
        self,
    ) -> Result<MetaPortfolio<Repository, Allocator, RiskManager, Statistic>, PortfolioError> {
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
            _statistic_marker: PhantomData,
        };

        // Persist initial state in the Repository
        portfolio.bootstrap_repository(
            self.starting_cash
                .ok_or(PortfolioError::BuilderIncomplete("starting_cash"))?,
            &self
                .markets
                .ok_or(PortfolioError::BuilderIncomplete("markets"))?,
            self.statistic_config
                .ok_or(PortfolioError::BuilderIncomplete("statistic_config"))?,
        )?;

        Ok(portfolio)
    }
}

/*----- */
// Impl MarketUpdater for MetaPortfolio
/*----- */
impl<Repository, Allocator, RiskManager, Statistic> MarketUpdater
    for MetaPortfolio<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
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
impl<Repository, Allocator, RiskManager, Statistic> OrderGenerator
    for MetaPortfolio<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
{
    fn generate_order(
        &mut self,
        signal: &Signal,
    ) -> Result<Option<Order<RequestOpen>>, PortfolioError> {
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

        let mut order = Order {
            exchange: signal.exchange,
            instrument: signal.instrument.clone(),
            client_order_id: ClientOrderId(Uuid::new_v4()),
            side: Side::from(*signal_decision),
            state: RequestOpen {
                kind: OrderKind::Limit,
                price: signal.market_meta.close,
                quantity: 0.0,
                decision: *signal_decision,
            },
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
impl<Repository, Allocator, RiskManager, Statistic> FillUpdater
    for MetaPortfolio<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
{
    fn update_from_fill(&mut self, fill: &FillEvent) -> Result<Vec<Event>, PortfolioError> {
        let mut generate_events = Vec::with_capacity(2);
        let mut balance = self.repository.get_balance(self.engine_id)?;

        let position_id = determine_position_id(self.engine_id, &fill.exchange, &fill.instrument);

        match self.repository.remove_position(&position_id)? {
            Some(mut position) => {
                let position_exit = position.exit(balance, fill)?;
                generate_events.push(Event::PositionExit(position_exit));

                balance.available += position.enter_value_gross
                    + position.realised_profit_loss
                    + position.enter_fees_total;
                balance.total += position.realised_profit_loss;

                // Update statistics for exited Position market
                let market_id = MarketId::new(&fill.exchange, &fill.instrument);

                let mut stats = self.repository.get_statistics(&market_id)?;
                stats.update(&position);

                // Persist exited Position & Updated Market statistics in Repository
                self.repository.set_statistics(market_id, stats)?;
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
// Impl StatisticHandler for MetaPortfolio
/*----- */
impl<Repository, Allocator, RiskManager, Statistic> StatisticHandler<Statistic>
    for MetaPortfolio<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
{
    fn set_statistics(
        &mut self,
        market_id: MarketId,
        statistic: Statistic,
    ) -> Result<(), RepositoryError> {
        self.repository.set_statistics(market_id, statistic)
    }

    fn get_statistics(&mut self, market_id: &MarketId) -> Result<Statistic, RepositoryError> {
        self.repository.get_statistics(market_id)
    }
}

/*----- */
// Impl PositionHandler for MetaPortfolio
/*----- */
impl<Repository, Allocator, RiskManager, Statistic> PositionHandler
    for MetaPortfolio<Repository, Allocator, RiskManager, Statistic>
where
    Repository: PositionHandler + BalanceHandler + StatisticHandler<Statistic>,
    Allocator: OrderAllocator,
    RiskManager: OrderEvaluator,
    Statistic: Initialiser + PositionSummariser,
{
    fn set_open_position(&mut self, position: Position) -> Result<(), RepositoryError> {
        self.repository.set_open_position(position)
    }

    fn get_open_position(
        &mut self,
        position_id: &PositionId,
    ) -> Result<Option<Position>, RepositoryError> {
        self.repository.get_open_position(position_id)
    }

    fn get_open_positions<'a, Markets: Iterator<Item = &'a Market>>(
        &mut self,
        _: Uuid,
        markets: Markets,
    ) -> Result<Vec<Position>, RepositoryError> {
        self.repository.get_open_positions(self.engine_id, markets)
    }

    fn remove_position(
        &mut self,
        position_id: &PositionId,
    ) -> Result<Option<Position>, RepositoryError> {
        self.repository.remove_position(position_id)
    }

    fn set_exited_position(&mut self, _: Uuid, position: Position) -> Result<(), RepositoryError> {
        self.repository
            .set_exited_position(self.engine_id, position)
    }

    fn get_exited_positions(&mut self, _: Uuid) -> Result<Vec<Position>, RepositoryError> {
        self.repository.get_exited_positions(self.engine_id)
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
