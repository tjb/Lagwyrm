//! Per-player input buffering keyed by tick. MODULE 1, GRADED.
//!
//! Clients stamp each command with the tick it should run on, a little in the
//! future, so it has time to cross the network. The server buffers commands
//! here and pulls exactly one command per player per tick.
//!
//! The struct layout below is a starting point. Change the private fields
//! however your design wants; keep the public API so the tests and the server
//! still compile.

use std::collections::BTreeMap;

use crate::command::{Command, Input};
use crate::ids::{PlayerId, Tick};

/// Why an input was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejected {
    /// The player was never registered.
    UnknownPlayer,
    /// The input's tick has already been simulated (it is `< next_tick`).
    Late { tick: Tick, next_tick: Tick },
    /// The input's tick is more than `window` ticks past `next_tick`.
    TooFarAhead { tick: Tick, next_tick: Tick },
    /// This player already has an input for that tick. The first one wins.
    Duplicate { tick: Tick },
}

/// Counters you maintain. Module 2's network simulator and the bot swarm both
/// report these, so make them right.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputStats {
    pub accepted: u64,
    pub late: u64,
    pub too_far_ahead: u64,
    pub duplicate: u64,
    pub unknown_player: u64,
    /// Ticks where a player had no input and their previous command was reused.
    pub repeated: u64,
}

#[derive(Clone, Debug, Default)]
#[allow(dead_code)] // remove once implemented
struct PlayerInputs {
    pending: BTreeMap<Tick, Command>,
    last: Command,
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // remove once implemented
pub struct InputQueue {
    window: u64,
    next_tick: Tick,
    players: BTreeMap<PlayerId, PlayerInputs>,
    stats: InputStats,
}

impl InputQueue {
    /// A queue that accepts inputs up to `window` ticks ahead of the next tick
    /// to be simulated.
    pub fn new(window: u64) -> InputQueue {
        InputQueue {
            window,
            next_tick: Tick::ZERO,
            players: BTreeMap::new(),
            stats: InputStats::default(),
        }
    }

    /// Starts tracking `player`. Their "last command" starts as [`Command::IDLE`].
    /// Registering twice is a no-op.
    pub fn register(&mut self, player: PlayerId) {
        self.players.entry(player).or_default();
    }

    pub fn unregister(&mut self, player: PlayerId) {
        self.players.remove(&player);
    }

    /// The next tick [`InputQueue::take`] expects to be called with.
    pub fn next_tick(&self) -> Tick {
        self.next_tick
    }

    pub fn stats(&self) -> InputStats {
        self.stats
    }

    /// Total buffered inputs across all players, for metrics and tests.
    pub fn pending_len(&self) -> usize {
        self.players.values().map(|p| p.pending.len()).sum()
    }

    /// Buffers one input.
    ///
    /// Accept iff the player is registered, `next_tick <= input.tick <=
    /// next_tick + window`, and the player has no input for that tick yet.
    /// Otherwise return the matching [`Rejected`] (check them in the order
    /// the enum lists them). Update [`InputStats`] either way.
    #[allow(unused_variables)] // remove once implemented
    pub fn push(&mut self, input: Input) -> Result<(), Rejected> {
        todo!("module 1: accept or reject an input")
    }

    /// Pulls the commands to simulate on `tick`: exactly one per registered
    /// player, sorted by `PlayerId` ascending.
    ///
    /// - A player with an input stamped `tick` gets that command, and it
    ///   becomes their "last command".
    /// - A player without one gets their last command again (count it in
    ///   `repeated`).
    /// - Buffered inputs for ticks `<= tick` are gone afterwards.
    /// - `next_tick()` becomes `tick + 1`.
    ///
    /// `tick` must be `>= next_tick()`. Skipping ahead is allowed; going back
    /// is a bug in the caller, so panic.
    #[allow(unused_variables)] // remove once implemented
    pub fn take(&mut self, tick: Tick) -> Vec<(PlayerId, Command)> {
        todo!("module 1: drain one tick of inputs")
    }
}
