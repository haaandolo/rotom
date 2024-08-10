pub mod live;
pub mod error;

pub trait MarketGenerator<Event> {
    fn next(&mut self) -> Feed<Event>;
} 

#[derive(Debug)]
pub enum Feed<Event> {
    Next(Event),
    UnHealthy,
    Finished
}