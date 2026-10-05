//! What a player asks the simulation to do on one tick.

use crate::ids::{PlayerId, Tick};

/// A player's intent for a single tick. Movement axes are `-1`, `0`, or `1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Command {
    pub move_x: i8,
    pub move_y: i8,
}

impl Command {
    /// Standing still.
    pub const IDLE: Command = Command {
        move_x: 0,
        move_y: 0,
    };

    /// Builds a command, clamping each axis into `-1..=1`. Clients are not
    /// trusted to send in-range values.
    pub fn new(move_x: i8, move_y: i8) -> Command {
        Command {
            move_x: move_x.clamp(-1, 1),
            move_y: move_y.clamp(-1, 1),
        }
    }
}

/// A command as it arrives at the server: who sent it and which tick it is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Input {
    pub player: PlayerId,
    /// The simulation tick this command should be applied on.
    pub tick: Tick,
    pub command: Command,
}
