# Module 3: Snapshots

**Status:** locked. Outline only until module 2 scores 80+.

## The concept

Each tick the server tells each client what the world looks like. Sending
full state is simple and far too big. The standard toolkit:

- **Bit-packing**: write fields with exactly as many bits as their range
  needs, not whole bytes.
- **Quantization**: send positions at the precision the client can see, not
  the precision the server simulates at, and know exactly how much error
  that introduces.
- **Delta compression against the last acked baseline**: encode only what
  changed since a snapshot the client has *confirmed* receiving (module 2's
  acks), so a lost packet never corrupts the client's view.

## Planned shape

- A `BitWriter` / `BitReader` pair and a quantizer for `Fx` coordinates.
- Per-client baseline tracking keyed by acked snapshot sequence.
- Snapshot encode/decode with full and delta paths, plus a fuzz-style
  round-trip test.
- Bots decode snapshots instead of reading `World` directly.

## Threads through

- **Replay:** snapshots are a pure function of world state and baseline, so
  a replay can regenerate and diff every snapshot.
- **Bot swarm:** reports bits per entity and delta hit rate.
- **Perf gate:** `bytes_per_client_per_sec` budget tightens to force deltas.

## References

1. Glenn Fiedler, *Snapshot Compression* and *Reading and Writing Packets*
   (Gaffer On Games, 2015 and 2016).
2. Fabien Sanglard, *Quake 3 Source Code Review: Network Model* (2012).
