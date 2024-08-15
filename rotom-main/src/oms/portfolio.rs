use rotom_data::event_models::market_event::{DataKind, MarketEvent};

use super::{allocator::OrderAllocator, error::PortfolioError, position::{determine_position_id, PositionUpdate}, repository::{BalanceHandler, PositionHandler}, risk::OrderEvaluator, MarketUpdater};

/*----- */
// Porfolio lego
/*----- */
pub struct PortfolioLego<Repository, Allocator, RiskManager> {
    pub repository: Repository,
    pub allocator: Allocator,
    pub risk: RiskManager,
}

/*----- */
// Metaportfolio
/*----- */
pub struct MetaPortfolio<Repository, Allocator, RiskManager> {
    pub repository: Repository,
    pub allocator: Allocator,
    pub risk: RiskManager,
}

impl<Repository, Allocator, RiskManager> MetaPortfolio<Repository, Allocator, RiskManager> {
    pub fn init(
        lego: PortfolioLego<Repository, Allocator, RiskManager>,
    ) -> Result<Self, PortfolioError> {
        let porfolio = Self {
            repository: lego.repository,
            allocator: lego.allocator,
            risk: lego.risk,
        };

        Ok(porfolio)
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
    RiskManager: OrderEvaluator
{
    fn update_from_market(
            &mut self,
            market: &MarketEvent<DataKind>
        ) -> Result<Option<PositionUpdate>, PortfolioError> {
        let position_id = determine_position_id(&market.exchange, &market.instrument);

        if let Some(mut position) = self.repository.get_open_positions(&position_id)? {
            if let Some(position_update) = position
        }

        Ok(None)
    }

}
