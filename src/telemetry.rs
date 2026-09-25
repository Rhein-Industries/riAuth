//! Process-local measurements. Labels are fixed by code, never by identities or URLs.
use serde_json::{Value, json};
use std::{
    fmt::Write,
    sync::atomic::{AtomicU64, Ordering::Relaxed},
    time::{Duration, Instant},
};

const BOUNDS: [u64; 8] = [
    100, 1_000, 10_000, 100_000, 500_000, 1_000_000, 5_000_000, 15_000_000,
];

#[derive(Default)]
pub struct Histogram {
    count: AtomicU64,
    micros: AtomicU64,
    buckets: [AtomicU64; 8],
}
impl Histogram {
    pub fn observe(&self, duration: Duration) {
        let micros = duration.as_micros().min(u64::MAX as u128) as u64;
        self.count.fetch_add(1, Relaxed);
        self.micros.fetch_add(micros, Relaxed);
        for (index, bound) in BOUNDS.iter().enumerate() {
            if micros <= *bound {
                self.buckets[index].fetch_add(1, Relaxed);
            }
        }
    }
    pub fn timer(&self) -> Timer<'_> {
        Timer {
            histogram: self,
            start: Instant::now(),
        }
    }
    pub fn snapshot(&self) -> Value {
        json!({"count":self.count.load(Relaxed),"seconds":self.micros.load(Relaxed) as f64 / 1_000_000.0})
    }
    pub fn render(&self, output: &mut String, name: &str, labels: &str) {
        let separator = if labels.is_empty() { "" } else { "," };
        for (index, bound) in BOUNDS.iter().enumerate() {
            writeln!(
                output,
                "{name}_bucket{{{labels}{separator}le=\"{}\"}} {}",
                *bound as f64 / 1_000_000.0,
                self.buckets[index].load(Relaxed)
            )
            .unwrap();
        }
        writeln!(
            output,
            "{name}_bucket{{{labels}{separator}le=\"+Inf\"}} {}",
            self.count.load(Relaxed)
        )
        .unwrap();
        let labels = if labels.is_empty() {
            String::new()
        } else {
            format!("{{{labels}}}")
        };
        writeln!(
            output,
            "{name}_count{labels} {}\n{name}_sum{labels} {}",
            self.count.load(Relaxed),
            self.micros.load(Relaxed) as f64 / 1_000_000.0
        )
        .unwrap();
    }
}
pub struct Timer<'a> {
    histogram: &'a Histogram,
    start: Instant,
}
impl Drop for Timer<'_> {
    fn drop(&mut self) {
        self.histogram.observe(self.start.elapsed());
    }
}

#[derive(Default)]
pub struct Telemetry {
    pub write_wait: Histogram,
    pub write_hold: Histogram,
    pub pool_wait: Histogram,
    pub signing: Histogram,
    pub password: Histogram,
    pub cleanup: Histogram,
    pub signing_errors: AtomicU64,
    pub cleanup_errors: AtomicU64,
    pub alert_delivery_errors: AtomicU64,
    pub scanned_records: AtomicU64,
    pub optimistic_conflicts: AtomicU64,
}
impl Telemetry {
    pub fn snapshot(&self) -> Value {
        json!({"write_wait":self.write_wait.snapshot(),"write_hold":self.write_hold.snapshot(),"pool_wait":self.pool_wait.snapshot(),"signing":self.signing.snapshot(),"password":self.password.snapshot(),"cleanup":self.cleanup.snapshot(),"signing_errors":self.signing_errors.load(Relaxed),"cleanup_errors":self.cleanup_errors.load(Relaxed),"alert_delivery_errors":self.alert_delivery_errors.load(Relaxed),"scanned_records":self.scanned_records.load(Relaxed),"optimistic_conflicts":self.optimistic_conflicts.load(Relaxed)})
    }
    pub fn render(&self, output: &mut String) {
        for (name, histogram) in [
            ("storage_write_wait", &self.write_wait),
            ("storage_write_hold", &self.write_hold),
            ("storage_pool_wait", &self.pool_wait),
            ("signing", &self.signing),
            ("password_verification", &self.password),
            ("cleanup", &self.cleanup),
        ] {
            writeln!(output, "# TYPE riauth_{name}_seconds histogram").unwrap();
            histogram.render(output, &format!("riauth_{name}_seconds"), "");
        }
        for (name, counter) in [
            ("signing_errors", &self.signing_errors),
            ("cleanup_errors", &self.cleanup_errors),
            ("alert_delivery_errors", &self.alert_delivery_errors),
            ("storage_scanned_records", &self.scanned_records),
            ("storage_optimistic_conflicts", &self.optimistic_conflicts),
        ] {
            writeln!(
                output,
                "# TYPE riauth_{name}_total counter\nriauth_{name}_total {}",
                counter.load(Relaxed)
            )
            .unwrap();
        }
    }
}
