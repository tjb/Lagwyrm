# Module 2: UDP transport

**Status:** locked. This is an outline. It is fleshed out (questions, skeleton,
tests, rubric) when module 1 scores 80+, against the code you wrote there.

## The concept

TCP's in-order, retransmit-everything delivery is wrong for real-time state:
one lost packet stalls everything behind it (head-of-line blocking), and by
the time a retransmit arrives the data is stale. Game servers build exactly
the reliability they need on top of UDP:

- **Sequence numbers** on every packet, with wraparound-safe comparison.
- **Ack bitfields**: each packet acks the latest sequence received plus a
  32-bit window of the ones before it, so acks are redundant and loss of an
  ack packet costs nothing.
- **Fragmentation and reassembly** for messages larger than the MTU, without
  letting a malicious client exhaust memory.
- **A network simulator** that sits between server and bots and injects
  loss, latency, jitter, duplication and reordering, deterministically from a
  seed, so tests and replays stay reproducible.

## Planned shape

- `lagwyrm-net`: packet header, sequence buffer, ack tracking, fragment
  assembler, `Transport` trait with a real UDP impl and a simulated one.
- Bots move from `Server::submit` to real packets through the transport.
- `BandwidthMeter` starts recording, so `bytes_per_client_per_sec` appears in
  swarm reports and the CI perf gate starts enforcing it.
- Your module 1 `InputQueue` stats become the main signal for how loss and
  jitter hurt the simulation.

## Threads through

- **Replay:** the network simulator is seeded; a swarm run with 5% loss
  replays identically.
- **Bot swarm:** gains `--loss`, `--latency-ms`, `--jitter-ms` flags.
- **Perf gate:** bytes per client is now measured and budgeted.

## References

1. Glenn Fiedler, *Reliable Ordered Messages* and *Packet Fragmentation and
   Reassembly* (Gaffer On Games, 2016).
2. Mark Frohnmayer and Tim Gift, *The TRIBES Engine Networking Model* (GDC 2000).
3. RFC 1982, *Serial Number Arithmetic*.
