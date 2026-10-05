# Module 1: The tick loop

**Status:** unlocked. Graded files: `crates/lagwyrm-sim/src/timestep.rs`,
`crates/lagwyrm-sim/src/input_queue.rs`, and `World::step` / `World::checksum`
in `crates/lagwyrm-sim/src/world.rs`.

## The concept

An authoritative server does not run "as fast as it can". It runs a
simulation in discrete **ticks** of exactly the same length, say 33.3ms at
30 Hz, and everything the world does happens on a tick boundary. Three ideas
make that work:

1. **Fixed timestep.** The OS wakes your loop at irregular times. An
   *accumulator* collects real elapsed time and pays it out as whole ticks of
   exactly `dt`. If the server falls badly behind, it runs a capped number of
   ticks and *drops* the rest rather than trying to catch up forever (the
   "spiral of death").
2. **Deterministic simulation.** `step(state, inputs) -> state'` is a pure
   function. Same state and same inputs give bit-identical results on any
   machine. That means integer or fixed-point math, a canonical iteration
   order, and no clocks or OS randomness inside the step. Lagwyrm uses Q16.16
   fixed-point (`Fx`) for that reason.
3. **Input queues.** Clients stamp each command with the tick it should run on,
   slightly in the future so it survives the trip. The server buffers commands
   per player and pulls exactly one per player per tick. A command that
   arrives after its tick is *late* and is dropped; a player with nothing
   buffered repeats their last command.

Put those together and you get the first of the three tools that run through
every module: **deterministic replay**. Record the inputs each tick consumed
plus a checksum of the resulting state, and you can rebuild any match bit for
bit and find the exact tick where two runs diverge.

## Why it matters in real game servers

- **Fairness and authority.** Players with fast machines or fast connections
  must not get more simulation. A fixed tick makes "what happened" independent
  of who sent packets when.
- **Debuggability.** Desync bugs are unreproducible without determinism. With
  it, a bug report is a replay file, and `lagwyrm-server replay` names the tick.
- **Everything later depends on it.** Snapshots (module 3) are taken per tick.
  Client prediction (module 4) re-simulates ticks and only works if the client
  and server step identically. Zone handoff (module 6) and crash recovery
  (module 7) both replay inputs against a known state.
- **Capacity.** The tick budget is the server's hard real-time deadline. The
  swarm's `tick_p99_us` metric, gated in CI, is how many players a zone can hold.

## What you build

| File | What | Tests |
|---|---|---|
| `crates/lagwyrm-sim/src/timestep.rs` | `FixedTimestep::advance`, `alpha` | `crates/lagwyrm-sim/tests/m01_timestep.rs` |
| `crates/lagwyrm-sim/src/input_queue.rs` | `InputQueue::push`, `take` | `crates/lagwyrm-sim/tests/m01_input_queue.rs` |
| `crates/lagwyrm-sim/src/world.rs` | `World::step`, `World::checksum` | `crates/lagwyrm-sim/tests/m01_world.rs` |
| (scaffolding, uses your code) | replay | `crates/lagwyrm-sim/tests/m01_replay.rs` |
| (scaffolding, uses your code) | server + bot swarm | `crates/lagwyrm-bot/tests/m01_swarm.rs` |

The doc comments on each `todo!()` are the spec. The tests check every clause.
You may restructure private fields (the input queue's layout is only a
suggestion); keep the public API so the server and tests compile.

## How to work through it

1. Read this file, then answer `QUESTIONS.md` in prose, in that file, before
   writing code. Understanding is 30% of the grade.
2. Read the scaffolding you will lean on: `fixed.rs`, `ids.rs`, `command.rs`,
   `replay.rs`, and `crates/lagwyrm-server/src/lib.rs` (`Server::tick` shows
   how your three pieces are called each tick).
3. Implement in this order, running one test file at a time:

   ```sh
   cargo test -p lagwyrm-sim --test m01_timestep
   cargo test -p lagwyrm-sim --test m01_input_queue
   cargo test -p lagwyrm-sim --test m01_world
   cargo test -p lagwyrm-sim --test m01_replay
   cargo test -p lagwyrm-bot --test m01_swarm
   ```

   Remove each `#[allow(unused_variables)] // remove once implemented` line as
   you fill in its function.
4. When everything is green, measure (10% of the grade). Run the swarm at
   three sizes and record what you see in the **Measurement** section of
   `QUESTIONS.md`:

   ```sh
   cargo run --release -p lagwyrm-bot -- swarm --bots 64   --ticks 3000
   cargo run --release -p lagwyrm-bot -- swarm --bots 256  --ticks 3000
   cargo run --release -p lagwyrm-bot -- swarm --bots 1024 --ticks 3000
   cargo run --release -p lagwyrm-bot -- swarm --record m01.replay
   cargo run --release -p lagwyrm-server -- replay m01.replay
   cargo run --release -p lagwyrm-server -- run --hz 60 --seconds 5
   cargo run --release -p lagwyrm-bot -- swarm --budget perf-budget.toml
   ```

5. Say **"grade module 1"** in the project.

Stuck? Ask for a hint. Hints escalate (a question, then a pointer to the
concept, then pseudocode) and each one is logged in `PROGRESS.md`. Nobody
will write the graded code for you.

## References

1. Glenn Fiedler, *Fix Your Timestep!* (Gaffer On Games, 2004). The
   accumulator, `alpha`, and the spiral of death.
   https://gafferongames.com/post/fix_your_timestep/
2. Glenn Fiedler, *Floating Point Determinism* (Gaffer On Games, 2010). Why
   "same code, same inputs" is not enough with floats.
   https://gafferongames.com/post/floating_point_determinism/
3. Timothy Ford, *Overwatch Gameplay Architecture and Netcode* (GDC 2017).
   Fixed command frames, input buffering on the server, and how the buffer
   size reacts to packet loss.
