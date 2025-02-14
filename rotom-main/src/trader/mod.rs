pub mod single_trader;
pub mod spot_arb_trader;

use crate::engine::Command;

pub trait TraderRun {
    fn receive_remote_command(&mut self) -> Option<Command>;

    fn subscribe_to_execution_manager(&mut self);

    fn run(self);
}
