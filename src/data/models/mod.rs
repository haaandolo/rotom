pub mod book;
pub mod trade;
pub mod level;
pub mod event;
pub mod subs;

use std::fmt::Debug;

pub trait SubKind
where
    Self: Debug + Clone,
{
    type Event: Debug + Send;
}