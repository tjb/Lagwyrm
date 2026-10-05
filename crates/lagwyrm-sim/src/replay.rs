//! Deterministic input replay. Scaffolding; this is a tool, not graded work.
//!
//! A [`Recording`] is everything needed to rebuild a match bit-for-bit: the
//! world config, the initial spawns, and for each tick the exact commands the
//! simulation consumed plus the checksum it produced. [`Recording::verify`]
//! re-runs it and reports the first tick where the checksum differs.
//!
//! Every later module keeps this working. If a change makes an old recording
//! diverge, either the change is a bug or the format needs a version bump.
//!
//! Module 1 limitation: spawns happen before the first frame only.

use std::fmt;
use std::path::Path;

use crate::command::Command;
use crate::fixed::Fx;
use crate::ids::{PlayerId, Tick};
use crate::world::{World, WorldConfig};

const MAGIC: &str = "lagwyrm-replay 1";

/// One simulated tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub tick: Tick,
    /// Exactly what was passed to [`World::step`].
    pub inputs: Vec<(PlayerId, Command)>,
    /// `World::checksum()` right after the step.
    pub checksum: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recording {
    pub config: WorldConfig,
    pub spawns: Vec<(PlayerId, Fx, Fx)>,
    pub frames: Vec<Frame>,
}

/// Where a replay stopped matching its recording.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Divergence {
    /// The world was on a different tick than the frame expected.
    TickMismatch { expected: Tick, actual: Tick },
    /// Same inputs, different state.
    Checksum {
        tick: Tick,
        expected: u64,
        actual: u64,
    },
}

impl fmt::Display for Divergence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Divergence::TickMismatch { expected, actual } => {
                write!(f, "frame is for {expected} but world is at {actual}")
            }
            Divergence::Checksum {
                tick,
                expected,
                actual,
            } => write!(
                f,
                "checksum diverged at {tick}: recorded {expected:016x}, replayed {actual:016x}"
            ),
        }
    }
}

impl std::error::Error for Divergence {}

impl Recording {
    pub fn new(config: WorldConfig) -> Recording {
        Recording {
            config,
            spawns: Vec::new(),
            frames: Vec::new(),
        }
    }

    pub fn record_spawn(&mut self, player: PlayerId, x: Fx, y: Fx) {
        assert!(
            self.frames.is_empty(),
            "module 1 recordings only support spawns before the first tick"
        );
        self.spawns.push((player, x, y));
    }

    pub fn record_frame(&mut self, tick: Tick, inputs: &[(PlayerId, Command)], checksum: u64) {
        self.frames.push(Frame {
            tick,
            inputs: inputs.to_vec(),
            checksum,
        });
    }

    /// A fresh world in the recording's starting state.
    pub fn initial_world(&self) -> World {
        let mut world = World::new(self.config);
        for &(player, x, y) in &self.spawns {
            world.spawn_at(player, x, y);
        }
        world
    }

    /// Replays every frame and checks each checksum. Returns the final world.
    pub fn verify(&self) -> Result<World, Divergence> {
        let mut world = self.initial_world();
        for frame in &self.frames {
            if world.tick() != frame.tick {
                return Err(Divergence::TickMismatch {
                    expected: frame.tick,
                    actual: world.tick(),
                });
            }
            world.step(&frame.inputs);
            let actual = world.checksum();
            if actual != frame.checksum {
                return Err(Divergence::Checksum {
                    tick: frame.tick,
                    expected: frame.checksum,
                    actual,
                });
            }
        }
        Ok(world)
    }

    /// Line-oriented text. Diffable, greppable, and readable in a pinch.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let c = &self.config;
        out.push_str(MAGIC);
        out.push('\n');
        out.push_str(&format!(
            "config {} {} {}\n",
            c.size.raw(),
            c.speed.raw(),
            c.half_extent.raw()
        ));
        for (p, x, y) in &self.spawns {
            out.push_str(&format!("spawn {} {} {}\n", p.0, x.raw(), y.raw()));
        }
        for f in &self.frames {
            out.push_str(&format!("frame {} {:016x}", f.tick.0, f.checksum));
            for (p, cmd) in &f.inputs {
                out.push_str(&format!(" {}:{}:{}", p.0, cmd.move_x, cmd.move_y));
            }
            out.push('\n');
        }
        out
    }

    pub fn from_text(text: &str) -> Result<Recording, String> {
        let mut lines = text.lines().enumerate();
        match lines.next() {
            Some((_, MAGIC)) => {}
            other => return Err(format!("not a lagwyrm replay: {other:?}")),
        }
        let mut config = None;
        let mut spawns = Vec::new();
        let mut frames = Vec::new();
        for (i, line) in lines {
            let err = |what: &str| format!("line {}: {what}: {line}", i + 1);
            let mut words = line.split_whitespace();
            match words.next() {
                None => continue,
                Some("config") => {
                    let n: Vec<i32> = words
                        .map(|w| w.parse().map_err(|_| err("bad number")))
                        .collect::<Result<_, _>>()?;
                    let [size, speed, half] = n[..] else {
                        return Err(err("config needs 3 numbers"));
                    };
                    config = Some(WorldConfig {
                        size: Fx::from_raw(size),
                        speed: Fx::from_raw(speed),
                        half_extent: Fx::from_raw(half),
                    });
                }
                Some("spawn") => {
                    let n: Vec<i64> = words
                        .map(|w| w.parse().map_err(|_| err("bad number")))
                        .collect::<Result<_, _>>()?;
                    let [p, x, y] = n[..] else {
                        return Err(err("spawn needs 3 numbers"));
                    };
                    spawns.push((
                        PlayerId(p as u32),
                        Fx::from_raw(x as i32),
                        Fx::from_raw(y as i32),
                    ));
                }
                Some("frame") => {
                    let tick = words
                        .next()
                        .and_then(|w| w.parse().ok())
                        .ok_or_else(|| err("bad tick"))?;
                    let checksum = words
                        .next()
                        .and_then(|w| u64::from_str_radix(w, 16).ok())
                        .ok_or_else(|| err("bad checksum"))?;
                    let mut inputs = Vec::new();
                    for w in words {
                        let parts: Vec<&str> = w.split(':').collect();
                        let [p, mx, my] = parts[..] else {
                            return Err(err("input must be player:x:y"));
                        };
                        let p = p.parse().map_err(|_| err("bad player"))?;
                        let mx = mx.parse().map_err(|_| err("bad move_x"))?;
                        let my = my.parse().map_err(|_| err("bad move_y"))?;
                        inputs.push((PlayerId(p), Command::new(mx, my)));
                    }
                    frames.push(Frame {
                        tick: Tick(tick),
                        inputs,
                        checksum,
                    });
                }
                Some(_) => return Err(err("unknown record")),
            }
        }
        Ok(Recording {
            config: config.ok_or("missing config line")?,
            spawns,
            frames,
        })
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path) -> std::io::Result<Recording> {
        let text = std::fs::read_to_string(path)?;
        Recording::from_text(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_format_round_trips() {
        let mut rec = Recording::new(WorldConfig::default());
        rec.record_spawn(PlayerId(7), Fx::from_int(3), Fx::from_ratio(5, 2));
        rec.record_frame(
            Tick(0),
            &[
                (PlayerId(7), Command::new(1, -1)),
                (PlayerId(9), Command::IDLE),
            ],
            0xdead_beef,
        );
        rec.record_frame(Tick(1), &[], 42);
        let back = Recording::from_text(&rec.to_text()).unwrap();
        assert_eq!(back, rec);
    }
}
