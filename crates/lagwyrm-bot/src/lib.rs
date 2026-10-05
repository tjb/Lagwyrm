//! Headless clients and the bot swarm harness. Scaffolding; not graded.
//!
//! In module 1 bots read the authoritative [`World`] directly and submit
//! inputs in-process. Later modules swap that for real packets and snapshots,
//! but the [`Bot`] trait and the [`SwarmReport`] metrics stay the same, so the
//! CI perf gate keeps working the whole way through.

pub mod budget;

use std::time::Duration;

use lagwyrm_server::{Server, ServerConfig};
use lagwyrm_sim::{Command, Fx, Input, PlayerId, SplitMix64, Tick, World};

/// A headless client that decides a command for a given tick.
pub trait Bot {
    fn player(&self) -> PlayerId;
    fn think(&mut self, tick: Tick, world: &World) -> Command;
}

/// Picks a random direction and holds it for a random number of ticks.
pub struct WanderBot {
    player: PlayerId,
    rng: SplitMix64,
    current: Command,
    hold: u32,
}

impl WanderBot {
    pub fn new(player: PlayerId, seed: u64) -> WanderBot {
        WanderBot {
            player,
            rng: SplitMix64::new(seed ^ (player.0 as u64).wrapping_mul(0x9E37_79B9)),
            current: Command::IDLE,
            hold: 0,
        }
    }
}

impl Bot for WanderBot {
    fn player(&self) -> PlayerId {
        self.player
    }

    fn think(&mut self, _tick: Tick, _world: &World) -> Command {
        if self.hold == 0 {
            let dx = self.rng.below(3) as i8 - 1;
            let dy = self.rng.below(3) as i8 - 1;
            self.current = Command::new(dx, dy);
            self.hold = 5 + self.rng.below(40) as u32;
        }
        self.hold -= 1;
        self.current
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SwarmConfig {
    pub bots: u32,
    /// Simulation ticks to run.
    pub ticks: u64,
    pub seed: u64,
    /// Bots stamp inputs this many ticks ahead of the server's next tick.
    pub input_delay: u64,
    /// Per-mille chance an input is stamped for a tick the server already
    /// simulated, as if it arrived late.
    pub late_per_mille: u64,
    /// Per-mille chance an input is lost entirely.
    pub loss_per_mille: u64,
    /// Server loop frames last `dt * U(1 - jitter, 1 + jitter)`, jitter in
    /// per-mille. Exercises the accumulator with uneven frames.
    pub frame_jitter_per_mille: u64,
    pub server: ServerConfig,
}

impl Default for SwarmConfig {
    fn default() -> SwarmConfig {
        SwarmConfig {
            bots: 64,
            ticks: 900,
            seed: 1,
            input_delay: 2,
            late_per_mille: 10,
            loss_per_mille: 10,
            frame_jitter_per_mille: 500,
            server: ServerConfig::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SwarmReport {
    pub bots: u32,
    pub ticks: u64,
    pub final_checksum: u64,
    pub tick_p50_us: f64,
    pub tick_p99_us: f64,
    pub tick_max_us: f64,
    pub dropped: Duration,
    pub inputs: lagwyrm_sim::InputStats,
    /// `None` until module 2 puts real packets on the wire.
    pub bytes_per_client_per_sec: Option<f64>,
}

impl SwarmReport {
    /// `key=value` lines, stable order, easy to diff and to parse in CI.
    pub fn to_lines(&self) -> String {
        let bpc = self
            .bytes_per_client_per_sec
            .map_or_else(|| "none".to_string(), |b| format!("{b:.1}"));
        format!(
            "bots={}\nticks={}\nfinal_checksum={:016x}\ntick_p50_us={:.1}\ntick_p99_us={:.1}\n\
             tick_max_us={:.1}\ndropped_ms={}\ninputs_accepted={}\ninputs_late={}\n\
             inputs_too_far_ahead={}\ninputs_duplicate={}\ninputs_repeated={}\n\
             bytes_per_client_per_sec={bpc}\n",
            self.bots,
            self.ticks,
            self.final_checksum,
            self.tick_p50_us,
            self.tick_p99_us,
            self.tick_max_us,
            self.dropped.as_millis(),
            self.inputs.accepted,
            self.inputs.late,
            self.inputs.too_far_ahead,
            self.inputs.duplicate,
            self.inputs.repeated,
        )
    }
}

/// Runs `config.bots` wander bots against an in-process server.
///
/// Fully deterministic in everything except the timing numbers: the same
/// config always produces the same `final_checksum` and the same recording.
pub struct Swarm {
    pub server: Server,
    bots: Vec<WanderBot>,
    /// Last tick each bot has submitted for, indexed like `bots`.
    submitted: Vec<Option<Tick>>,
    rng: SplitMix64,
    config: SwarmConfig,
}

impl Swarm {
    pub fn new(config: SwarmConfig) -> Swarm {
        let mut server = Server::new(config.server);
        let mut rng = SplitMix64::new(config.seed);
        let size = config.server.world.size.raw() as u64;
        let mut bots = Vec::new();
        for i in 0..config.bots {
            let player = PlayerId(i);
            let x = Fx::from_raw(rng.below(size) as i32);
            let y = Fx::from_raw(rng.below(size) as i32);
            server.join(player, x, y);
            bots.push(WanderBot::new(player, config.seed));
        }
        Swarm {
            server,
            submitted: vec![None; bots.len()],
            bots,
            rng,
            config,
        }
    }

    /// Each bot submits one input per tick, up to `next_tick + input_delay`,
    /// as a client running its own clock slightly ahead would.
    fn submit_inputs(&mut self) {
        let next = self.server.world().tick();
        let target = next.plus(self.config.input_delay);
        for (bot, last) in self.bots.iter_mut().zip(self.submitted.iter_mut()) {
            let mut t = last.map_or(next, Tick::next);
            while t <= target {
                let command = bot.think(t, self.server.world());
                *last = Some(t);
                let stamp = t;
                t = t.next();
                if self.rng.chance(self.config.loss_per_mille, 1000) {
                    continue;
                }
                let stamp = if next.0 > 0 && self.rng.chance(self.config.late_per_mille, 1000) {
                    Tick(next.0 - 1)
                } else {
                    stamp
                };
                // Rejections are counted in InputStats; bots don't care.
                let _ = self.server.submit(Input {
                    player: bot.player(),
                    tick: stamp,
                    command,
                });
            }
        }
    }

    pub fn run(mut self) -> (SwarmReport, Server) {
        let dt = self.server.timestep().dt().as_nanos() as u64;
        let jitter = self.config.frame_jitter_per_mille.min(999);
        while self.server.world().tick().0 < self.config.ticks {
            self.submit_inputs();
            let lo = dt * (1000 - jitter) / 1000;
            let span = dt * 2 * jitter / 1000 + 1;
            let frame = lo + self.rng.below(span);
            // Never overshoot the requested tick count.
            let remaining = self.config.ticks - self.server.world().tick().0;
            let frame = frame.min(remaining * dt);
            self.server.pump(Duration::from_nanos(frame));
        }
        let s = &self.server;
        let sim_seconds = s.world().tick().0 as f64 / s.config().tick_hz as f64;
        let report = SwarmReport {
            bots: self.config.bots,
            ticks: s.world().tick().0,
            final_checksum: s.world().checksum(),
            tick_p50_us: s.tick_stats().percentile_us(50.0),
            tick_p99_us: s.tick_stats().percentile_us(99.0),
            tick_max_us: s.tick_stats().max_us(),
            dropped: s.dropped_time(),
            inputs: s.input_stats(),
            bytes_per_client_per_sec: s.bandwidth().bytes_per_client_per_sec(sim_seconds),
        };
        (report, self.server)
    }
}
