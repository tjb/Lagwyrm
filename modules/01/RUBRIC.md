# Module 1 rubric

Score out of 100. 80 or more unlocks module 2.

## Correctness (40)

| Points | Criterion |
|---|---|
| 30 | `cargo test --workspace` passes. Partial credit is proportional to passing module 1 tests (`m01_*`), rounded down. |
| 5 | No panics on any valid input sequence: no overflow on long runs, no `unwrap` on data from clients, the only panic is `take` going back in time. |
| 5 | `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` are clean, and every `#[allow(unused_variables)]` scaffold marker is gone. |

## Understanding (30)

Five points per question in `QUESTIONS.md`.

| Points | What earns them |
|---|---|
| 5 | Correct, with a concrete scenario (tick, entities, symptom) and the mechanism behind it. |
| 3 | Correct mechanism, but generic or missing the "what breaks" consequence. |
| 1 | Restates the definition. |
| 0 | Missing or wrong. |

Questions 1 and 6 also need their "argue the other side" or "what do you do"
part for full marks; question 4 needs the counter-argument.

## Design (20)

| Points | Criterion |
|---|---|
| 6 | `FixedTimestep::advance` uses integer `Duration` arithmetic, never floats, for the accumulator; the conservation invariant is obvious from reading it. |
| 6 | `InputQueue`: data structure fits the access pattern (insert by tick, drain everything `<= tick`), stale inputs cannot pile up, stats are updated in one place, and rejection order is explicit. |
| 6 | `World::step`: canonical order is enforced by construction rather than by sorting the caller's slice and hoping; no per-entity allocations; the O(n²) blocking check is acknowledged (a comment, or a note in QUESTIONS.md) since module 5 fixes it. |
| 2 | `checksum`: fixed, documented algorithm with a fixed byte order; covers tick, ids, owners and both raw coordinates. |

## Measurement (10)

| Points | Criterion |
|---|---|
| 4 | Swarm numbers for 64, 256 and 1024 bots, run in `--release`. |
| 3 | Scaling explained against the actual implementation, with a bots-per-tick-budget estimate. |
| 2 | At least one `--delay` experiment with the effect on `inputs_repeated` / `inputs_late` explained. |
| 1 | Replay round trip shown (`swarm --record` then `server replay`, checksums match). |
