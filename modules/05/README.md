# Module 5: Interest management

**Status:** locked. Outline only until module 4 scores 80+.

## The concept

An MMO zone has thousands of entities; each client can see and afford a
few dozen per packet. Interest management decides who hears about what:

- **Spatial grid**: bucket entities by cell so "what is near me" is O(nearby),
  not O(world). It also fixes module 1's O(n²) blocking check.
- **Priority accumulator**: every entity's priority for a client grows each
  tick it is not sent (faster if near or important) and resets when sent, so
  nothing starves.
- **Per-client bandwidth budget**: fill each packet in priority order until
  the byte budget is spent.

## Planned shape

- A uniform grid with insert, move and query, used by both `World::step` and
  snapshot building.
- A per-client priority table and a packet builder that respects a byte
  budget.
- Swarm scenarios with crowding: everyone walks to one point.

## Threads through

- **Replay:** relevance sets are deterministic, so a replay reproduces
  exactly which entities each client was sent.
- **Bot swarm:** scales to thousands of bots; reports entities-per-packet
  and starvation (max ticks an in-range entity went unsent).
- **Perf gate:** tick p99 must now stay flat-ish as bots grow, not quadratic.

## References

1. Glenn Fiedler, *State Synchronization* (Gaffer On Games, 2015): the
   priority accumulator.
2. Mark Frohnmayer and Tim Gift, *The TRIBES Engine Networking Model* (GDC
   2000): the ghost manager and scoping.
3. Epic Games, *Replication Graph* (Unreal Engine documentation).
