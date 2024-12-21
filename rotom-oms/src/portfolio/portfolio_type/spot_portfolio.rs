use chrono::Utc;
use rotom_data::{
    error::SocketError,
    event_models::market_event::{DataKind, MarketEvent},
    shared::subscription_models::ExchangeId,
    ExchangeAssetId,
};
use rotom_strategy::{Decision, Signal, SignalForceExit, SignalStrength};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    event::Event,
    exchange::{
        binance::binance_client::BinancePrivateData, poloniex::poloniex_client::PoloniexPrivateData,
    },
    execution::FillEvent,
    model::{
        account_data::{
            AccountDataBalance, AccountDataBalanceDelta, AccountDataOrder, OrderStatus,
        },
        balance::{determine_balance_id, SpotBalanceId},
        order::{OrderEvent, OrderState},
        OrderKind, Side,
    },
    portfolio::{
        allocator::{spot_arb_allocator::SpotArbAllocator, OrderAllocator},
        error::PortfolioError,
        persistence::spot_in_memory::SpotInMemoryRepository,
        position::{
            self, determine_position_id, Position, PositionEnterer, PositionExiter, PositionUpdate,
            PositionUpdater,
        },
        position2::Position2,
    },
};

use super::{FillUpdater, MarketUpdater, OrderGenerator};

#[derive(Debug)]
pub struct SpotPortfolio {
    pub engine_id: Uuid,
    pub exchanges: Vec<ExchangeId>,
    pub repository: SpotInMemoryRepository,
    pub allocator: SpotArbAllocator,
}

impl SpotPortfolio {
    pub fn new(
        engine_id: Uuid,
        exchanges: Vec<ExchangeId>,
        repository: SpotInMemoryRepository,
        allocator: SpotArbAllocator,
    ) -> Self {
        Self {
            engine_id,
            exchanges,
            repository,
            allocator,
        }
    }

    pub async fn init(mut self) -> Result<SpotPortfolio, SocketError> {
        for exchange in self.exchanges.iter() {
            let exchange_balance = match exchange {
                ExchangeId::BinanceSpot => {
                    let balance = BinancePrivateData::new().get_balance_all().await?;
                    let asset_balance: Vec<AccountDataBalance> = balance.into();
                    asset_balance
                }
                ExchangeId::PoloniexSpot => {
                    let balance = PoloniexPrivateData::new().get_balance_all().await?;
                    let asset_balance: Vec<AccountDataBalance> = balance.into();
                    asset_balance
                }
            };

            for asset_balance in exchange_balance.into_iter() {
                self.repository
                    .set_balance(SpotBalanceId::from(&asset_balance), asset_balance.balance)
                    .unwrap(); // todo
            }
        }

        Ok(self)
    }

    pub fn update_balance(&mut self, balance_update: &AccountDataBalance) {
        self.repository.update_balance(balance_update)
    }

    pub fn update_balance_delta(&mut self, balance_update: &AccountDataBalanceDelta) {
        self.repository.update_balance_delta(balance_update)
    }

    // todo: adjust buffer?
    pub fn no_cash_to_enter_new_position(
        &mut self,
        balance_id: SpotBalanceId,
        buy_sell_amount: f64,
    ) -> Result<bool, PortfolioError> {
        self.repository
            .get_balance(&balance_id)
            .map(|balance| balance.total * 0.98 < buy_sell_amount) // give 2% buffer to account for fee
            .map_err(PortfolioError::RepositoryInteraction)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////////////////

    // OrderGenerator trait
    pub fn generate_order(
        &mut self,
        signal: &Signal,
    ) -> Result<Option<OrderEvent>, PortfolioError> {
        let position = self.repository.get_open_position(&ExchangeAssetId::from((
            &signal.exchange,
            &signal.instrument,
        )))?;

        let (signal_decision, signal_strength) =
            match parse_signal_decision2(&position, &signal.signals) {
                None => return Ok(None),
                Some(net_signal) => net_signal,
            };

        let mut order = OrderEvent {
            order_request_time: Utc::now(),
            exchange: signal.exchange,
            instrument: signal.instrument.clone(),
            client_order_id: None,
            market_meta: signal.market_meta,
            decision: *signal_decision,
            quantity: 0.0,
            order_kind: OrderKind::Limit,
            order_status: OrderStatus::New,
            state: OrderState::RequestOpen,
            filled_gross: 0.0,
        };

        self.allocator
            .allocate_order2(&mut order, position, *signal_strength);

        let balance_id = determine_balance_id(&signal.instrument.quote, &signal.exchange);

        if position.is_none()
            && self.no_cash_to_enter_new_position(balance_id, order.get_dollar_value())?
        {
            return Ok(None);
        }

        Ok(Some(order))
    }

    pub fn generate_exit_order2(
        &mut self,
        _signal: SignalForceExit,
    ) -> Result<Option<OrderEvent>, PortfolioError> {
        unimplemented!()
    }

    // MarketUpdater trait
    pub fn update_from_market2(&mut self) {
        unimplemented!()
    }

    // FillUpdater trait
    pub fn update_from_fill2(
        &mut self,
        account_data: &AccountDataOrder,
        order: &OrderEvent,
    ) -> Result<(), PortfolioError> {
        // self.repository.remove_posisition removes a position if it is open
        let position_id = ExchangeAssetId::from((&order.exchange, &order.instrument));
        match self.repository.remove_position(&position_id)? {
            Some(mut position) => {}
            // If no position is open for the current Exchange & asset combo, we should enter a position
            None => {
                let position = Position2::enter(self.engine_id, account_data, order);
                self.repository.set_open_position(position)?;
            }
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////
// Traits for Spot Portfolio - Update later
////////////////////////////////////////////////////////////////////////////////////////////////////////
impl MarketUpdater for SpotPortfolio {
    fn update_from_market(
        &mut self,
        market: &MarketEvent<DataKind>,
    ) -> Result<Option<PositionUpdate>, PortfolioError> {
        unimplemented!()
        // let position_id =
        //     determine_position_id(self.engine_id, &market.exchange, &market.instrument);

        // if let Some(position) = self.repository.get_open_position_mut(&position_id)? {
        //     if let Some(position_update) = position.update(market) {
        //         return Ok(Some(position_update));
        //     }
        // }

        // Ok(None)
    }
}

impl OrderGenerator for SpotPortfolio {
    fn generate_order(&mut self, signal: &Signal) -> Result<Option<OrderEvent>, PortfolioError> {
        unimplemented!()
        // let position_id =
        //     determine_position_id(self.engine_id, &signal.exchange, &signal.instrument);
        // let position = self.repository.get_open_position(&position_id)?;

        // let (signal_decision, signal_strength) =
        //     match parse_signal_decision(&position, &signal.signals) {
        //         None => return Ok(None),
        //         Some(net_signal) => net_signal,
        //     };

        // let mut order = OrderEvent {
        //     order_request_time: Utc::now(),
        //     exchange: signal.exchange,
        //     instrument: signal.instrument.clone(),
        //     client_order_id: None,
        //     market_meta: signal.market_meta,
        //     decision: *signal_decision,
        //     quantity: 0.0,
        //     order_kind: OrderKind::Limit,
        //     order_status: None,
        //     state: OrderState::RequestOpen,
        //     filled_gross: 0.0,
        // };

        // self.allocator
        //     .allocate_order(&mut order, position, *signal_strength);

        // let balance_id = determine_balance_id(&signal.instrument.quote, &signal.exchange);

        // if position.is_none()
        //     && self.no_cash_to_enter_new_position(balance_id, order.get_dollar_value())?
        // {
        //     return Ok(None);
        // }

        // Ok(Some(order))
    }

    fn generate_exit_order(
        &mut self,
        _signal: SignalForceExit,
    ) -> Result<Option<OrderEvent>, PortfolioError> {
        unimplemented!()
    }
}

impl FillUpdater for SpotPortfolio {
    fn update_from_fill(&mut self, fill: &FillEvent) -> Result<Vec<Event>, PortfolioError> {
        unimplemented!()
        // let mut generate_events = Vec::with_capacity(2);

        // // Get required balance and position ids
        // let base_asset_balance_id = determine_balance_id(&fill.instrument.base, &fill.exchange);
        // let quote_asset_balance_id = determine_balance_id(&fill.instrument.quote, &fill.exchange);

        // // Get balances of base and quote asset
        // let mut base_asset_balance = self.repository.get_balance(&base_asset_balance_id)?;
        // let mut quote_asset_balance = self.repository.get_balance(&quote_asset_balance_id)?;

        // let position_id = determine_position_id(self.engine_id, &fill.exchange, &fill.instrument);

        // match self.repository.remove_position(&position_id)? {
        //     // Exit scenario
        //     Some(mut position) => {
        //         println!("########## REMOVE POSITION ##########");
        //         let position_exit =
        //             position.exit_spot(&mut base_asset_balance, &mut quote_asset_balance, fill)?;
        //         generate_events.push(Event::PositionExit(position_exit));
        //     }
        //     // Enter scenario
        //     None => {
        //         println!("######### REMOVE POSITION NONE ##########");
        //         let position = Position::enter(self.engine_id, fill)?;
        //         generate_events.push(Event::PositionNew(position.clone()));

        //         // Update quote asset balance
        //         quote_asset_balance.total += position.calculate_quote_asset_enter_price();

        //         // Update base asset balance
        //         base_asset_balance.total += fill.quantity;

        //         self.repository.set_open_position(position)?;
        //     }
        // }

        // // todo: do we need this still? probs not
        // self.repository
        //     .set_balance(quote_asset_balance_id, quote_asset_balance)?;

        // self.repository
        //     .set_balance(base_asset_balance_id, base_asset_balance)?;

        // println!(">>>>> SELF.PORTFOLIO <<<<<<");
        // println!("{:#?}", self);

        // Ok(generate_events)
    }
}

/*----- */
// Parse Signal Decision
/*----- */

pub fn parse_signal_decision2<'a>(
    position: &'a Option<&Position2>,
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
