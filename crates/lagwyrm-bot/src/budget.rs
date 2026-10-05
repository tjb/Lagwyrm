//! The CI perf gate. Reads `perf-budget.toml` (flat `key = number` lines only)
//! and checks a [`SwarmReport`] against it.

use crate::SwarmReport;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Budget {
    pub tick_p99_us: Option<f64>,
    pub bytes_per_client_per_sec: Option<f64>,
}

impl Budget {
    pub fn parse(text: &str) -> Result<Budget, String> {
        let mut b = Budget::default();
        for (i, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() || line.starts_with('[') {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("line {}: expected key = value", i + 1))?;
            let value: f64 = value
                .trim()
                .parse()
                .map_err(|_| format!("line {}: not a number", i + 1))?;
            match key.trim() {
                "tick_p99_us" => b.tick_p99_us = Some(value),
                "bytes_per_client_per_sec" => b.bytes_per_client_per_sec = Some(value),
                other => return Err(format!("line {}: unknown budget {other}", i + 1)),
            }
        }
        Ok(b)
    }

    /// One message per exceeded budget. Empty means the gate passes. A metric
    /// the report doesn't have yet (bytes before module 2) is skipped.
    pub fn check(&self, report: &SwarmReport) -> Vec<String> {
        let mut failures = Vec::new();
        if let Some(max) = self.tick_p99_us
            && report.tick_p99_us > max
        {
            failures.push(format!(
                "tick_p99_us {:.1} exceeds budget {max:.1}",
                report.tick_p99_us
            ));
        }
        if let (Some(max), Some(actual)) = (
            self.bytes_per_client_per_sec,
            report.bytes_per_client_per_sec,
        ) && actual > max
        {
            failures.push(format!(
                "bytes_per_client_per_sec {actual:.1} exceeds budget {max:.1}"
            ));
        }
        failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_toml() {
        let b = Budget::parse(
            "[swarm]\ntick_p99_us = 500 # comment\n\nbytes_per_client_per_sec=8000\n",
        )
        .unwrap();
        assert_eq!(b.tick_p99_us, Some(500.0));
        assert_eq!(b.bytes_per_client_per_sec, Some(8000.0));
        assert!(Budget::parse("nope = 1").is_err());
    }
}
