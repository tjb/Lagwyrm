//! Deterministic simulation core for Lagwyrm.
//!
//! Everything in this crate must be a pure function of its inputs: no wall
//! clock, no OS randomness, no iteration over unordered collections, no
//! floating point in the authoritative state. If two machines feed the same
//! recording into this crate they must produce bit-identical worlds.
//!
//! Module 1 lives here. The files with `todo!()` in them are yours:
//!
//! - [`timestep`]: the fixed-timestep accumulator.
//! - [`input_queue`]: per-player input buffering keyed by tick.
//! - [`world`]: `World::step` and `World::checksum`.
//!
//! Everything else ([`fixed`], [`ids`], [`command`], [`rng`], [`replay`]) is
//! scaffolding you can read, use, and change if your design needs it.

pub mod command;
pub mod fixed;
pub mod ids;
pub mod input_queue;
pub mod replay;
pub mod rng;
pub mod timestep;
pub mod world;

pub use command::{Command, Input};
pub use fixed::Fx;
pub use ids::{EntityId, PlayerId, Tick};
pub use input_queue::{InputQueue, InputStats, Rejected};
pub use replay::{Divergence, Frame, Recording};
pub use rng::SplitMix64;
pub use timestep::{FixedTimestep, Steps};
pub use world::{Entity, World, WorldConfig};
