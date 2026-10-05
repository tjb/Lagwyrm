//! `lagwyrm-bot swarm`: run N bots against an in-process server, print
//! metrics, optionally save a replay and enforce the perf budget.
//!
//!     cargo run --release -p lagwyrm-bot -- swarm --bots 256 --ticks 3000
//!     cargo run --release -p lagwyrm-bot -- swarm --record match.replay
//!     cargo run --release -p lagwyrm-bot -- swarm --budget perf-budget.toml

use std::path::Path;
use std::process::ExitCode;

use lagwyrm_bot::budget::Budget;
use lagwyrm_bot::{Swarm, SwarmConfig};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) != Some("swarm") {
        eprintln!(
            "usage: lagwyrm-bot swarm [--bots N] [--ticks N] [--seed N] [--delay N]\n\
             \x20                        [--late-per-mille N] [--loss-per-mille N]\n\
             \x20                        [--record FILE] [--budget FILE]"
        );
        return ExitCode::from(2);
    }
    let args = &args[1..];
    let num = |name: &str, default: u64| -> u64 {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    };
    let text = |name: &str| -> Option<&String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
    };

    let defaults = SwarmConfig::default();
    let config = SwarmConfig {
        bots: num("--bots", defaults.bots as u64) as u32,
        ticks: num("--ticks", defaults.ticks),
        seed: num("--seed", defaults.seed),
        input_delay: num("--delay", defaults.input_delay),
        late_per_mille: num("--late-per-mille", defaults.late_per_mille),
        loss_per_mille: num("--loss-per-mille", defaults.loss_per_mille),
        ..defaults
    };

    let (report, server) = Swarm::new(config).run();
    print!("{}", report.to_lines());

    if let Some(path) = text("--record") {
        let rec = server.recording().expect("server records by default");
        if let Err(e) = rec.save(Path::new(path)) {
            eprintln!("cannot write {path}: {e}");
            return ExitCode::FAILURE;
        }
        eprintln!("replay saved to {path}");
    }

    if let Some(path) = text("--budget") {
        let budget = match std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|t| Budget::parse(&t))
        {
            Ok(b) => b,
            Err(e) => {
                eprintln!("bad budget file {path}: {e}");
                return ExitCode::FAILURE;
            }
        };
        let failures = budget.check(&report);
        if !failures.is_empty() {
            for f in failures {
                eprintln!("PERF GATE FAILED: {f}");
            }
            return ExitCode::FAILURE;
        }
        eprintln!("perf gate passed");
    }
    ExitCode::SUCCESS
}
