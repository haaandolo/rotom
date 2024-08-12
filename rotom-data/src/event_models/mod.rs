pub mod event_book;
pub mod event_trade;
pub mod market_event;

use std::fmt::Debug;

pub trait SubKind
where
    Self: Debug + Clone,
{
    type Event: Debug + Send;
}
