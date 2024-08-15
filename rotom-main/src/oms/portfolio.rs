use chrono::Utc;
use futures::stream::All;
use rotom_data::event_models::market_event::{DataKind, MarketEvent};
use uuid::Uuid;

use super::{
    allocator::OrderAllocator, error::PortfolioError, position::{determine_position_id, PositionUpdate, PositionUpdater}, repository::{self, error::RepositoryError, BalanceHandler, PositionHandler}, risk::OrderEvaluator, Balance, MarketUpdater
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
    pub repository: Repository,
    pub allocator: Allocator,
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
        let porfolio = Self {
            engine_id: lego.engine_id,
            repository: lego.repository,
            allocation_manager: lego.allocator,
            risk_manager: lego.risk,
        };

        Ok(porfolio)
    }

    pub fn bootstrap_repository(&mut self, starting_cash: f64) -> Result<(), PortfolioError> {
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
            // &self
            //     .markets
            //     .ok_or(PortfolioError::BuilderIncomplete("markets"))?,
            // self.statistic_config
            //     .ok_or(PortfolioError::BuilderIncomplete("statistic_config"))?,
        )?;

        Ok(portfolio)
    }
}

/*----- */
// Impl Market Updater for MetaPortfolio
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
