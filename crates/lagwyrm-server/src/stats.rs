//! Tick timing. Measures how long the server spends inside each tick, which is
//! what the CI perf gate's `tick_p99_us` budget is checked against.

use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct TickStats {
    samples_ns: Vec<u64>,
}

impl TickStats {
    pub fn record(&mut self, d: Duration) {
        self.samples_ns.push(d.as_nanos() as u64);
    }

    pub fn count(&self) -> usize {
        self.samples_ns.len()
    }

    /// Nearest-rank percentile in microseconds, `p` in `0.0..=100.0`.
    pub fn percentile_us(&self, p: f64) -> f64 {
        if self.samples_ns.is_empty() {
            return 0.0;
        }
        let mut sorted = self.samples_ns.clone();
        sorted.sort_unstable();
        let rank = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
        sorted[rank.clamp(1, sorted.len()) - 1] as f64 / 1_000.0
    }

    pub fn mean_us(&self) -> f64 {
        if self.samples_ns.is_empty() {
            return 0.0;
        }
        self.samples_ns.iter().sum::<u64>() as f64 / self.samples_ns.len() as f64 / 1_000.0
    }

    pub fn max_us(&self) -> f64 {
        self.samples_ns.iter().copied().max().unwrap_or(0) as f64 / 1_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles() {
        let mut s = TickStats::default();
        for us in 1..=100 {
            s.record(Duration::from_micros(us));
        }
        assert_eq!(s.percentile_us(50.0), 50.0);
        assert_eq!(s.percentile_us(99.0), 99.0);
        assert_eq!(s.percentile_us(100.0), 100.0);
        assert_eq!(s.max_us(), 100.0);
    }
}
