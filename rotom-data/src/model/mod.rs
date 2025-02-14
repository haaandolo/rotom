pub mod event_book;
pub mod event_book_snapshot;
pub mod event_trade;
pub mod market_event;
pub mod network_info;
pub mod ticker_info;

/*----- */
// Event Kind
/*----- */
#[derive(Debug, Clone, Copy)]
pub enum EventKind {
    OrderBook,
    Trade,
}

/*----- */
// SubKind trait
/*----- */
pub trait SubKind
where
    Self: std::fmt::Debug + Clone,
{
    const EVENTKIND: EventKind;
    type Event: std::fmt::Debug + Send;
}
