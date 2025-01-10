pub mod single_trader;
pub mod spot_arb_trader;

use crate::engine::Command;

pub trait TraderRun {
    fn receive_remote_command(&mut self) -> Option<Command>;

    fn run(self);
}
