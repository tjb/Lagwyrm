# Module 7: Failure and scale

**Status:** locked. Outline only until module 6 scores 80+.

## The concept

Zone servers crash. Disks are slow. Ten thousand players log in at once.

- **Write-behind persistence**: the tick loop never waits on a database.
  Dirty state is batched and flushed asynchronously, with a bounded window
  of acceptable loss that you choose and can state.
- **Zone crash recovery**: restore a zone from its last persisted state plus
  the input log since (module 1's replay, now load-bearing), and re-home its
  players through the gateway.
- **Bot swarm load test**: thousands of bots across zones, with chaos
  (killed zones, network partitions), and a report of what degraded and how
  fast it recovered.

## Planned shape

- A persistence trait with an in-memory "slow disk" implementation that
  injects latency and failures.
- Checkpoint plus input-log recovery, verified by checksum against the
  pre-crash run.
- A swarm scenario runner and a final load-test report.

## Threads through

- **Replay:** is the recovery mechanism itself.
- **Bot swarm:** becomes the load test.
- **Perf gate:** adds recovery time and tick p99 during a flush.

## References

1. Joe Armstrong, *Making reliable distributed systems in the presence of
   software errors* (PhD thesis, KTH, 2003).
2. Martin Kleppmann, *Designing Data-Intensive Applications* (O'Reilly,
   2017), chapters 5 and 11 (Replication, Stream Processing).
