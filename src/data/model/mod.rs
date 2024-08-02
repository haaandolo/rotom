pub mod event_book;
pub mod event_trade;
pub mod level;
pub mod event;
pub mod subs;

use std::fmt::Debug;
use serde::de::DeserializeOwned;

pub trait SubKind
where
    Self: Debug + Clone,
{
    type Event: Debug + Send + DeserializeOwned;
}