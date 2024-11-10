use std::collections::HashMap;

use chrono::Utc;
use hmac::digest::consts::False;
use rotom_data::{
    error::SocketError,
    event_models::market_event::{DataKind, MarketEvent},
    exchange,
    shared::subscription_models::ExchangeId,
};
use rotom_strategy::{Decision, Signal, SignalForceExit, SignalStrength};
use uuid::Uuid;

use crate::{
    event::Event,
    exchange::{
        binance::binance_client::BinancePrivateData, poloniex::poloniex_client::PoloniexPrivateData,
    },
    execution::FillEvent,
    portfolio::{
        allocator::{
            default_allocator::DefaultAllocator, spot_arb_allocator::SpotArbAllocator,
            OrderAllocator,
        },
        determine_balance_id,
        error::PortfolioError,
        persistence::in_memory2::InMemoryRepository2,
        position::{
            determine_position_id, Position, PositionEnterer, PositionUpdate, PositionUpdater, Side,
        },
        AssetBalance, BalanceId2, OrderEvent, OrderType,
    },
};

use super::{FillUpdater, MarketUpdater, OrderGenerator};

#[derive(Debug)]
pub struct ArbPortfolio {
    pub engine_id: Uuid,
    pub exchanges: Vec<ExchangeId>,
    pub repository: InMemoryRepository2,
    pub allocator: SpotArbAllocator,
}

impl ArbPortfolio {
    pub fn new(
        engine_id: Uuid,
        exchanges: Vec<ExchangeId>,
        repository: InMemoryRepository2,
        allocator: SpotArbAllocator,
    ) -> Self {
        Self {
            engine_id,
            exchanges,
            repository,
            allocator,
        }
    }

    pub async fn init(mut self) -> Result<ArbPortfolio, SocketError> {
        for exchange in self.exchanges.iter() {
            let exchange_balance = match exchange {
                ExchangeId::BinanceSpot => {
                    let balance = BinancePrivateData::new().get_balance_all().await?;
                    let asset_balance: Vec<AssetBalance> = balance.into();
                    asset_balance
                }
                ExchangeId::PoloniexSpot => {
                    let balance = PoloniexPrivateData::new().get_balance_all().await?;
                    let asset_balance: Vec<AssetBalance> = balance.into();
                    asset_balance
                }
            };

            for asset_balance in exchange_balance.into_iter() {
                self.repository
                    .set_balance(BalanceId2::from(&asset_balance), asset_balance.balance)
                    .unwrap(); // todo
            }
        }

        Ok(self)
    }

    // todo
    pub fn no_cash_to_enter_new_position(
        &mut self,
        balance_id: BalanceId2,
        buy_sell_amount: f64,
    ) -> Result<bool, PortfolioError> {
        self.repository
            .get_balance(balance_id)
            .map(|balance| balance.available < buy_sell_amount)
            .map_err(PortfolioError::RepositoryInteraction)
    }
}

impl MarketUpdater for ArbPortfolio {
    fn update_from_market(
        &mut self,
        market: &MarketEvent<DataKind>,
    ) -> Result<Option<PositionUpdate>, PortfolioError> {
        let position_id =
            determine_position_id(self.engine_id, &market.exchange, &market.instrument);

        if let Some(position) = self.repository.get_open_position_mut(&position_id)? {
            // println!("##############################");
            // println!("position in arb portfolio --> {:#?}", position);
            if let Some(position_update) = position.update(market) {
                return Ok(Some(position_update));
            }
        }

        Ok(None)
    }
}

impl OrderGenerator for ArbPortfolio {
    fn generate_order(&mut self, signal: &Signal) -> Result<Option<OrderEvent>, PortfolioError> {
        let position_id =
            determine_position_id(self.engine_id, &signal.exchange, &signal.instrument);
        let position = self.repository.get_open_position(&position_id)?;

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

        self.allocator
            .allocate_order(&mut order, position, *signal_strength);

        let balance_id = determine_balance_id(&signal.instrument.quote, &signal.exchange);

        if position.is_none()
            && self.no_cash_to_enter_new_position(balance_id, order.get_dollar_value())?
        {
            return Ok(None);
        }

        Ok(Some(order))
    }

    fn generate_exit_order(
        &mut self,
        signal: SignalForceExit,
    ) -> Result<Option<OrderEvent>, PortfolioError> {
        unimplemented!()
    }
}

impl FillUpdater for ArbPortfolio {
    fn update_from_fill(&mut self, fill: &FillEvent) -> Result<Vec<Event>, PortfolioError> {
        let mut generate_events = Vec::new();
        let position = Position::enter(self.engine_id, fill)?;
        generate_events.push(Event::PositionNew(position.clone()));
        self.repository.set_open_position(position)?;

        println!(">>>>> SELF.REPOSITORY <<<<<<");
        println!("{:#?}", self.repository);

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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct Posit {
        pub value: f64,
    }

    #[test]
    fn testing() {
        let mut testing_hash = HashMap::new();
        let value = Posit { value: 40.0 };
        testing_hash.insert("op", value);
        println!("before {:#?}", testing_hash);

        let pos = testing_hash.get_mut("op").unwrap();
        pos.value = 69.0;
        println!("after {:#?}", testing_hash);
    }
}
