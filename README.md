# Lagwyrm

An authoritative MMO game server in Rust, built one module at a time to
learn netcode and distributed world state.

Claude is the tutor and grader here, not the implementer: it builds the
scaffolding and grades the work; the graded parts are written by hand. See
[`CLAUDE.md`](CLAUDE.md) for the rules and [`PROGRESS.md`](PROGRESS.md) for
the grade log.

## Modules

| # | Topic | Status |
|---|---|---|
| [01](modules/01) | Tick loop: fixed timestep, deterministic simulation, input queues, bots | **unlocked** |
| [02](modules/02) | UDP transport: sequence numbers, ack bitfields, fragmentation, network simulator | locked |
| [03](modules/03) | Snapshots: bit-packing, quantization, delta compression | locked |
| [04](modules/04) | Prediction, reconciliation, interpolation, lag compensation | locked |
| [05](modules/05) | Interest management: spatial grid, priority accumulator, bandwidth budget | locked |
| [06](modules/06) | Sharding: zone servers, gateway, entity handoff, ghosts | locked |
| [07](modules/07) | Failure and scale: write-behind persistence, crash recovery, load test | locked |

Each module scores out of 100 (correctness 40, understanding 30, design 20,
measurement 10). 80 unlocks the next.

## Layout

```
crates/
  lagwyrm-sim/      deterministic simulation (module 1 lives here)
  lagwyrm-net/      transport (module 2 on)
  lagwyrm-server/   authoritative server binary + replay verifier
  lagwyrm-bot/      headless bots, swarm harness, perf gate
modules/0N/         README, QUESTIONS, RUBRIC per module
perf-budget.toml    CI perf gate budgets
```

## Tools used in every module

```sh
# Bot swarm: N bots against an in-process server, prints metrics
cargo run --release -p lagwyrm-bot -- swarm --bots 256 --ticks 3000

# Deterministic replay: record a match, then verify it tick by tick
cargo run --release -p lagwyrm-bot -- swarm --record match.replay
cargo run --release -p lagwyrm-server -- replay match.replay

# Perf gate (what CI runs once tests pass)
cargo run --release -p lagwyrm-bot -- swarm --budget perf-budget.toml
```

CI is red until module 1's `todo!()`s are implemented. That is expected:
the failing tests are the assignment.
