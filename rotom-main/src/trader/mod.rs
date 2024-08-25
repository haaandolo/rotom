use crate::engine::Command;

pub mod arb_trader;
pub mod single_trader;

/*----- */
// Trader trait
/*----- */
pub trait TraderRun {
    fn receive_remote_command(&mut self) -> Option<Command>;

    fn run(self);
}
