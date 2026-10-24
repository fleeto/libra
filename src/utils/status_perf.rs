//! Opt-in, process-local counters for status investigation. No paths or content
//! are recorded. The CLI explicitly writes a new private diagnostic file.
use std::{collections::BTreeMap, fs::OpenOptions, io, sync::Mutex, time::Instant};

#[derive(Default, serde::Serialize)]
struct Metric {
    calls: u64,
    elapsed_ns: u128,
}
static METRICS: Mutex<BTreeMap<&'static str, Metric>> = Mutex::new(BTreeMap::new());

pub struct Span {
    name: &'static str,
    start: Instant,
}
impl Span {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        if let Ok(mut metrics) = METRICS.lock() {
            let value = metrics.entry(self.name).or_default();
            value.calls += 1;
            value.elapsed_ns += self.start.elapsed().as_nanos();
        }
    }
}
#[cfg(test)]
thread_local! {
    static TEST_COUNTS: std::cell::RefCell<BTreeMap<&'static str, u64>> = const { std::cell::RefCell::new(BTreeMap::new()) };
}
pub fn count(name: &'static str, amount: u64) {
    #[cfg(test)]
    TEST_COUNTS.with(|counts| *counts.borrow_mut().entry(name).or_default() += amount);
    if let Ok(mut metrics) = METRICS.lock() {
        metrics.entry(name).or_default().calls += amount;
    }
}
pub fn flush() -> io::Result<()> {
    let Some(path) = std::env::var_os("LIBRA_STATUS_PERF_FILE") else {
        return Ok(());
    };
    let metrics = METRICS
        .lock()
        .map_err(|_| io::Error::other("status diagnostic counters poisoned"))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    serde_json::to_writer(file, &*metrics).map_err(io::Error::other)
}
#[cfg(test)]
pub(crate) fn test_count(name: &str) -> u64 {
    TEST_COUNTS.with(|counts| counts.borrow().get(name).copied().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_perf_counts_requests_separately_from_hashes() {
        count("test.requests", 3);
        count("test.hashes", 7);
        let span = Span::new("test.phase");
        drop(span);
        let metrics = METRICS.lock().unwrap();
        assert_eq!(metrics["test.requests"].calls, 3);
        assert_eq!(metrics["test.hashes"].calls, 7);
        assert_eq!(metrics["test.phase"].calls, 1);
        let json = serde_json::to_value(&*metrics).unwrap();
        assert!(json["test.phase"]["elapsed_ns"].is_number());
    }
}
