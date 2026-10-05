//! Fixed-timestep accumulator. MODULE 1, GRADED.
//!
//! The server's outer loop wakes up whenever the OS lets it, measures how much
//! wall-clock time passed, and asks this type how many simulation ticks to run.
//! The simulation itself only ever sees whole ticks of exactly `dt`.
//!
//! Read modules/01/README.md and answer modules/01/QUESTIONS.md before you
//! fill in the `todo!()`s.

use std::time::Duration;

/// What one call to [`FixedTimestep::advance`] decided.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Steps {
    /// How many ticks the caller must simulate now.
    pub ticks: u32,
    /// Wall-clock time thrown away because running it would exceed
    /// `max_steps`. Non-zero means the server is falling behind.
    pub dropped: Duration,
}

#[derive(Clone, Debug)]
pub struct FixedTimestep {
    dt: Duration,
    max_steps: u32,
    accumulator: Duration,
}

impl FixedTimestep {
    /// A timestep running at `hz` ticks per second that will never ask for more
    /// than `max_steps` ticks from a single `advance` call.
    ///
    /// `dt` is `1s / hz` rounded down to the nanosecond, so at 60 Hz it is
    /// 16_666_666 ns, not 16.666...ms.
    pub fn new(hz: u32, max_steps: u32) -> FixedTimestep {
        assert!(hz > 0, "tick rate must be positive");
        assert!(max_steps > 0, "max_steps must be positive");
        FixedTimestep {
            dt: Duration::from_nanos(1_000_000_000 / hz as u64),
            max_steps,
            accumulator: Duration::ZERO,
        }
    }

    /// The length of one tick.
    pub fn dt(&self) -> Duration {
        self.dt
    }

    pub fn max_steps(&self) -> u32 {
        self.max_steps
    }

    /// Time fed in that has not yet been turned into a tick. Always `< dt`
    /// after `advance` returns.
    pub fn accumulated(&self) -> Duration {
        self.accumulator
    }

    /// Feeds `elapsed` wall-clock time in and returns how many ticks to run.
    ///
    /// Contract (the tests check all of it):
    /// - Time is never lost or invented: across any sequence of calls,
    ///   `sum(ticks) * dt + sum(dropped) + accumulated()` equals the total
    ///   `elapsed` fed in.
    /// - `ticks <= max_steps`. When more whole ticks are owed than that, run
    ///   `max_steps`, and report the extra *whole* ticks' worth of time as
    ///   `dropped`. The sub-tick remainder stays in the accumulator.
    /// - `accumulated() < dt` when this returns.
    #[allow(unused_variables)] // remove once implemented
    pub fn advance(&mut self, elapsed: Duration) -> Steps {
        todo!("module 1: fixed-timestep accumulator")
    }

    /// How far we are between the last simulated tick and the next one, in
    /// `[0.0, 1.0)`. A renderer would blend the last two states by this much.
    /// The server only uses it for metrics, but you will need it in module 4.
    pub fn alpha(&self) -> f32 {
        todo!("module 1: interpolation fraction")
    }
}
