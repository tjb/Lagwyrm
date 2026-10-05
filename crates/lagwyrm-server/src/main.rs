//! `lagwyrm-server`: run the tick loop against the real clock, or verify a
//! replay recording.
//!
//!     cargo run -p lagwyrm-server -- run --hz 30 --seconds 5
//!     cargo run -p lagwyrm-server -- replay match.replay

use std::path::Path;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use lagwyrm_server::{Server, ServerConfig};
use lagwyrm_sim::Recording;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("run") => run(&args[1..]),
        Some("replay") => match args.get(1) {
            Some(path) => replay(Path::new(path)),
            None => usage(),
        },
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: lagwyrm-server run [--hz N] [--seconds N]");
    eprintln!("       lagwyrm-server replay <file>");
    ExitCode::from(2)
}

fn flag(args: &[String], name: &str, default: u64) -> u64 {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// A real-time loop with no players: watch how the accumulator copes with the
/// OS scheduler. Bots live in `lagwyrm-bot`; see its `swarm` command.
fn run(args: &[String]) -> ExitCode {
    let hz = flag(args, "--hz", 30) as u32;
    let seconds = flag(args, "--seconds", 5);
    let mut server = Server::new(ServerConfig {
        tick_hz: hz,
        record: false,
        ..ServerConfig::default()
    });
    let started = Instant::now();
    let mut last = started;
    let mut frames = 0u64;
    while started.elapsed() < Duration::from_secs(seconds) {
        let now = Instant::now();
        server.pump(now - last);
        last = now;
        frames += 1;
        let ts = server.timestep();
        std::thread::sleep(ts.dt().saturating_sub(ts.accumulated()));
    }
    let wall = started.elapsed().as_secs_f64();
    let ticks = server.world().tick().0;
    println!(
        "ran {ticks} ticks in {wall:.3}s ({:.2} Hz, target {hz}), {frames} loop iterations",
        ticks as f64 / wall
    );
    println!("dropped time: {:?}", server.dropped_time());
    ExitCode::SUCCESS
}

fn replay(path: &Path) -> ExitCode {
    let rec = match Recording::load(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cannot load {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    match rec.verify() {
        Ok(world) => {
            println!(
                "ok: {} frames, {} entities, final checksum {:016x}",
                rec.frames.len(),
                world.len(),
                world.checksum()
            );
            ExitCode::SUCCESS
        }
        Err(d) => {
            eprintln!("DIVERGED: {d}");
            ExitCode::FAILURE
        }
    }
}
