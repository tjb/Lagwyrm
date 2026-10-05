# Module 1 questions

Answer each in prose, right under the question, before you write the code.
A paragraph or two each is plenty. Concrete scenarios beat definitions: name
the tick, the player, the symptom. You may revise answers after coding; say
what changed your mind if you do.

## 1. The spiral of death

Suppose `advance` had no `max_steps` cap, and a GC-like pause (a slow disk
write, a page fault storm) makes one frame take 2 seconds at 30 Hz. Walk
through what the next few loop iterations do. Now suppose instead that each
tick simply costs more than `dt` to compute. What breaks, and why does a cap
fix one case but not the other? When the cap does kick in and time is
dropped, what does a connected player experience, and why is dropping time
better than letting the simulation clock fall permanently behind wall time?

> _Your answer._

## 2. Variable timesteps

What breaks if the server skips the accumulator and just calls
`world.step(elapsed)` with whatever time passed since the last frame? Think
about the blocking rule (an entity moving 2 units in one big step versus 1
unit twice), about replay, and about two servers simulating the same zone.

> _Your answer._

## 3. Order

`World::step` must give the same result whatever order `inputs` arrive in,
and entities move in ascending `EntityId` order. What breaks if you instead
apply commands in the order packets arrived, or iterate a `HashMap`?
Describe a concrete two-entity scenario where the outcome flips, and name two
later features (see the module list in the root `README.md`) that would
silently go wrong because of it.

> _Your answer._

## 4. Floats

Lagwyrm's authoritative state is Q16.16 fixed-point, not `f32`. What breaks
if positions were `f32`, given the same Rust source on every machine? Name at
least two concrete sources of divergence. Then argue the other side: where in
a game server is `f32` perfectly fine, and what is the rule that separates
the two?

> _Your answer._

## 5. The input window and late inputs

Clients stamp inputs `input_delay` ticks ahead, and the queue accepts at most
`window` ticks ahead. What breaks if `input_delay` is too small? Too large?
What breaks if `window` is too small, or unbounded? When a player's input is
missing, the queue repeats their last command: for movement that is usually
right. Give an example of a command type where repeating is wrong, and what
you would do instead.

> _Your answer._

## 6. Checksums

What breaks if `checksum` uses `std::collections::hash_map::DefaultHasher`,
or `#[derive(Hash)]` over a struct containing a `HashMap`? What breaks if it
combines per-entity hashes with XOR or addition? In production, a client's
checksum for tick 48_213 disagrees with the server's. What do you log, and
what do you do for that player right now?

> _Your answer._

## Measurement

Fill this in after the tests pass. Paste the swarm output (or the relevant
lines) for 64, 256 and 1024 bots, then answer:

- How does `tick_p99_us` scale with bot count, and why, given how you
  implemented `World::step`?
- At 30 Hz, roughly how many bots fit in one tick's budget on your machine
  before you would start dropping time? Show the arithmetic.
- What does `inputs_repeated` tell you about the default `--delay 2` and the
  simulated late and lost inputs? Try another `--delay` and report the
  change.
- Did `lagwyrm-server replay` verify your recording? What was the final
  checksum, and did it match the swarm's `final_checksum`?

> _Your numbers and analysis._
