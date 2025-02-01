pub mod event_book;
pub mod event_book_snapshot;
pub mod event_trade;
pub mod market_event;
pub mod network_info;
pub mod ticker_info;

pub trait SubKind
where
    Self: std::fmt::Debug + Clone,
{
    type Event: std::fmt::Debug + Send;
}
