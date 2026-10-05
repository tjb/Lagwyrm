# Module 4: Prediction, reconciliation, interpolation, lag compensation

**Status:** locked. Outline only until module 3 scores 80+.

## The concept

With 100ms of round trip, a client that waits for the server feels like
mud. Four techniques hide latency without giving up authority:

- **Client-side prediction**: the client runs the same deterministic step
  (your module 1 `World::step`) on its own inputs immediately.
- **Reconciliation**: when an authoritative snapshot arrives, rewind to it
  and re-simulate the inputs the server has not yet acknowledged.
- **Entity interpolation**: other players are rendered a little in the past,
  blended between two snapshots (this is where `FixedTimestep::alpha` returns).
- **Lag compensation**: when a client fires, the server rewinds other
  entities to where *that client* saw them, within a bounded history.

## Planned shape

- A client-side prediction buffer of unacked inputs, and a reconcile step
  that measures and reports misprediction distance.
- An interpolation buffer with configurable delay.
- A server-side history ring buffer and a hitscan check against rewound state.
- Bots run full prediction so the swarm exercises it.

## Threads through

- **Replay:** client prediction is checked by replaying the client's own
  input log against server snapshots.
- **Bot swarm:** reports misprediction rate and correction distance under
  simulated latency.
- **Perf gate:** history buffer memory and rewind cost are budgeted.

## References

1. Gabriel Gambetta, *Fast-Paced Multiplayer* (series, gabrielgambetta.com).
2. Yahn Bernier, *Latency Compensating Methods in Client/Server In-game
   Protocol Design and Optimization* (Valve, 2001).
3. Timothy Ford, *Overwatch Gameplay Architecture and Netcode* (GDC 2017).
