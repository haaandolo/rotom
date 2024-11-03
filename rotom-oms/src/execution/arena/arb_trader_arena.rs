use crate::exchange::ExecutionClient2;

pub trait ArbTraderArena {
    type ExchangeOne: ExecutionClient2;
    type ExchangeTwo: ExecutionClient2;

}