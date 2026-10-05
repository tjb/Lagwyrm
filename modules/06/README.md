# Module 6: Sharding

**Status:** locked. Outline only until module 5 scores 80+.

## The concept

One process cannot hold the whole world. Split it into zones, each owned by
one zone server, behind a gateway clients connect to:

- **Gateway**: owns client connections and routes inputs and snapshots to
  whichever zone owns that player's entity.
- **Entity handoff**: when an entity crosses a boundary, authority moves from
  one zone to another without duplicating it, losing it, or losing its inputs
  in flight. Deterministic ticks give you a precise "as of tick N" for the
  transfer.
- **Ghost entities**: read-only copies of entities near a boundary, mirrored
  into the neighbouring zone so players can see and block each other across it.

## Planned shape

- Multiple zone servers in one process first (channels), then separate
  processes over the module 2 transport.
- A handoff protocol with explicit states and a test that kills messages at
  each step.
- Ghost replication using module 3 snapshots and module 5 interest.

## Threads through

- **Replay:** a multi-zone recording replays per zone, and handoffs line up
  by tick.
- **Bot swarm:** bots that deliberately pace along zone borders.
- **Perf gate:** per-zone tick p99 and gateway forwarding cost.

## References

1. CCP Games, *Introducing Time Dilation* (EVE Online dev blog, 2011): what
   to do when one shard is overloaded.
2. Martin Kleppmann, *Designing Data-Intensive Applications* (O'Reilly,
   2017), chapter 6 (Partitioning).
3. Cloud Imperium Games, *Server Meshing* talks (CitizenCon 2022 and 2023):
   entity authority across servers at scale.
