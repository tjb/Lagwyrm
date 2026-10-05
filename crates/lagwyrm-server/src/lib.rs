//! The authoritative server: glues the timestep, input queue, world, replay
//! recorder, and metrics together. Scaffolding; not graded in module 1.
//!
//! In module 1 everything runs in-process: bots call [`Server::submit`]
//! directly. Module 2 puts `lagwyrm-net` between them.

pub mod stats;

use std::time::{Duration, Instant};

use lagwyrm_net::BandwidthMeter;
use lagwyrm_sim::{
    Command, EntityId, FixedTimestep, Fx, Input, InputQueue, InputStats, PlayerId, Recording,
    Rejected, Steps, World, WorldConfig,
};

pub use stats::TickStats;

#[derive(Clone, Copy, Debug)]
pub struct ServerConfig {
    pub tick_hz: u32,
    /// Most ticks one `pump` call may run before dropping time.
    pub max_steps: u32,
    /// How far ahead of the current tick clients may stamp inputs.
    pub input_window: u64,
    pub world: WorldConfig,
    /// Keep a [`Recording`] of every tick for replay.
    pub record: bool,
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            tick_hz: 30,
            max_steps: 5,
            input_window: 32,
            world: WorldConfig::default(),
            record: true,
        }
    }
}

pub struct Server {
    config: ServerConfig,
    world: World,
    inputs: InputQueue,
    timestep: FixedTimestep,
    recording: Option<Recording>,
    stats: TickStats,
    bandwidth: BandwidthMeter,
    dropped: Duration,
}

impl Server {
    pub fn new(config: ServerConfig) -> Server {
        Server {
            config,
            world: World::new(config.world),
            inputs: InputQueue::new(config.input_window),
            timestep: FixedTimestep::new(config.tick_hz, config.max_steps),
            recording: config.record.then(|| Recording::new(config.world)),
            stats: TickStats::default(),
            bandwidth: BandwidthMeter::default(),
            dropped: Duration::ZERO,
        }
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Adds a player and their entity. Module 1 only supports joining before
    /// the first tick when recording is on.
    pub fn join(&mut self, player: PlayerId, x: Fx, y: Fx) -> EntityId {
        self.inputs.register(player);
        if let Some(rec) = &mut self.recording {
            rec.record_spawn(player, x, y);
        }
        self.world.spawn_at(player, x, y)
    }

    pub fn submit(&mut self, input: Input) -> Result<(), Rejected> {
        self.inputs.push(input)
    }

    /// Runs exactly one simulation tick.
    pub fn tick(&mut self) {
        let start = Instant::now();
        let tick = self.world.tick();
        let commands: Vec<(PlayerId, Command)> = self.inputs.take(tick);
        self.world.step(&commands);
        let checksum = self.world.checksum();
        self.stats.record(start.elapsed());
        if let Some(rec) = &mut self.recording {
            rec.record_frame(tick, &commands, checksum);
        }
    }

    /// Feeds wall-clock time in and runs however many ticks it is worth.
    pub fn pump(&mut self, elapsed: Duration) -> Steps {
        let steps = self.timestep.advance(elapsed);
        self.dropped += steps.dropped;
        for _ in 0..steps.ticks {
            self.tick();
        }
        steps
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn timestep(&self) -> &FixedTimestep {
        &self.timestep
    }

    pub fn input_stats(&self) -> InputStats {
        self.inputs.stats()
    }

    pub fn tick_stats(&self) -> &TickStats {
        &self.stats
    }

    /// Total wall-clock time thrown away because the server fell behind.
    pub fn dropped_time(&self) -> Duration {
        self.dropped
    }

    pub fn bandwidth(&self) -> &BandwidthMeter {
        &self.bandwidth
    }

    pub fn recording(&self) -> Option<&Recording> {
        self.recording.as_ref()
    }
}
