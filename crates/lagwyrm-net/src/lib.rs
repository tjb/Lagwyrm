//! Transport layer for Lagwyrm. Empty until module 2 is unlocked.
//!
//! Module 2 adds UDP sockets, sequence numbers, ack bitfields, fragmentation,
//! and a network simulator (loss, latency, jitter) here. In module 1 bots talk
//! to the server in-process, so the only thing that lives here today is the
//! bandwidth meter the bot swarm and the CI perf gate already report from.

use std::collections::BTreeMap;

/// Counts bytes sent to and received from each client.
///
/// Module 1 records nothing, so `bytes_per_client_per_sec` reports `None` and
/// the perf gate skips that budget. From module 2 on, the transport calls
/// [`BandwidthMeter::sent`] and [`BandwidthMeter::received`] for every packet.
#[derive(Clone, Debug, Default)]
pub struct BandwidthMeter {
    per_client: BTreeMap<u32, ClientBytes>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClientBytes {
    pub sent: u64,
    pub received: u64,
}

impl BandwidthMeter {
    pub fn sent(&mut self, client: u32, bytes: usize) {
        self.per_client.entry(client).or_default().sent += bytes as u64;
    }

    pub fn received(&mut self, client: u32, bytes: usize) {
        self.per_client.entry(client).or_default().received += bytes as u64;
    }

    pub fn client(&self, client: u32) -> ClientBytes {
        self.per_client.get(&client).copied().unwrap_or_default()
    }

    /// Mean server-to-client bytes per second, or `None` if nothing was sent.
    pub fn bytes_per_client_per_sec(&self, sim_seconds: f64) -> Option<f64> {
        let total: u64 = self.per_client.values().map(|c| c.sent).sum();
        if total == 0 || self.per_client.is_empty() || sim_seconds <= 0.0 {
            return None;
        }
        Some(total as f64 / self.per_client.len() as f64 / sim_seconds)
    }
}
