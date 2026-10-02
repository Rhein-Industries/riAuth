//! Process-local measurements. Labels are fixed by code, never by identities or URLs.
use serde_json::{Value, json};
use std::{
    cell::Cell,
    fmt::Write,
    sync::atomic::{AtomicU64, Ordering::Relaxed},
    time::{Duration, Instant},
};

const BOUNDS: [u64; 8] = [
    100, 1_000, 10_000, 100_000, 500_000, 1_000_000, 5_000_000, 15_000_000,
];
/// Records per storage scan.
const ROWS: [u64; 7] = [0, 1, 16, 128, 1_024, 8_192, 65_536];

fn render_buckets(
    output: &mut String,
    name: &str,
    labels: &str,
    bounds: impl Iterator<Item = (String, u64)>,
    count: u64,
    sum: String,
) {
    let separator = if labels.is_empty() { "" } else { "," };
    for (bound, value) in bounds {
        writeln!(
            output,
            "{name}_bucket{{{labels}{separator}le=\"{bound}\"}} {value}"
        )
        .unwrap();
    }
    writeln!(
        output,
        "{name}_bucket{{{labels}{separator}le=\"+Inf\"}} {count}"
    )
    .unwrap();
    let labels = if labels.is_empty() {
        String::new()
    } else {
        format!("{{{labels}}}")
    };
    writeln!(
        output,
        "{name}_count{labels} {count}\n{name}_sum{labels} {sum}"
    )
    .unwrap();
}

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
    pub fn count(&self) -> u64 {
        self.count.load(Relaxed)
    }
    pub fn micros(&self) -> u64 {
        self.micros.load(Relaxed)
    }
    pub fn snapshot(&self) -> Value {
        json!({"count":self.count.load(Relaxed),"seconds":self.micros.load(Relaxed) as f64 / 1_000_000.0})
    }
    pub fn render(&self, output: &mut String, name: &str, labels: &str) {
        render_buckets(
            output,
            name,
            labels,
            BOUNDS.iter().enumerate().map(|(index, bound)| {
                (
                    (*bound as f64 / 1_000_000.0).to_string(),
                    self.buckets[index].load(Relaxed),
                )
            }),
            self.count.load(Relaxed),
            (self.micros.load(Relaxed) as f64 / 1_000_000.0).to_string(),
        );
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

/// Distribution of record counts, e.g. rows materialized by one scan.
#[derive(Default)]
pub struct Sizes {
    count: AtomicU64,
    sum: AtomicU64,
    buckets: [AtomicU64; 7],
}
impl Sizes {
    pub fn observe(&self, size: u64) {
        self.count.fetch_add(1, Relaxed);
        self.sum.fetch_add(size, Relaxed);
        for (index, bound) in ROWS.iter().enumerate() {
            if size <= *bound {
                self.buckets[index].fetch_add(1, Relaxed);
            }
        }
    }
    pub fn count(&self) -> u64 {
        self.count.load(Relaxed)
    }
    pub fn sum(&self) -> u64 {
        self.sum.load(Relaxed)
    }
    pub fn snapshot(&self) -> Value {
        json!({"count":self.count(),"sum":self.sum()})
    }
    pub fn render(&self, output: &mut String, name: &str, labels: &str) {
        render_buckets(
            output,
            name,
            labels,
            ROWS.iter()
                .enumerate()
                .map(|(index, bound)| (bound.to_string(), self.buckets[index].load(Relaxed))),
            self.count(),
            self.sum().to_string(),
        );
    }
}

/// Callers currently inside a section, and the most seen at once since start.
/// Two relaxed atomics; never a lock.
#[derive(Default)]
pub struct Occupancy {
    current: AtomicU64,
    peak: AtomicU64,
}
impl Occupancy {
    pub fn enter(&self) {
        let now = self.current.fetch_add(1, Relaxed).saturating_add(1);
        self.peak.fetch_max(now, Relaxed);
    }
    pub fn leave(&self) {
        self.current.fetch_sub(1, Relaxed);
    }
    pub fn current(&self) -> u64 {
        self.current.load(Relaxed)
    }
    pub fn peak(&self) -> u64 {
        self.peak.load(Relaxed)
    }
    /// Returns the peak and restarts it from the current value, for per-phase
    /// characterization. Exported metrics report the peak since the last call.
    pub fn take_peak(&self) -> u64 {
        self.peak.swap(self.current(), Relaxed)
    }
    fn snapshot(&self) -> Value {
        json!({"current":self.current(),"peak":self.peak()})
    }
}

/// Work competing for storage. A blocking task declares its activity; everything
/// else, including HTTP, LDAP and RADIUS requests, is foreground.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Activity {
    #[default]
    Foreground,
    Maintenance,
    Provisioning,
    Mail,
    LogoutDelivery,
    SsfDelivery,
}
impl Activity {
    pub const ALL: [Activity; 6] = [
        Activity::Foreground,
        Activity::Maintenance,
        Activity::Provisioning,
        Activity::Mail,
        Activity::LogoutDelivery,
        Activity::SsfDelivery,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Activity::Foreground => "foreground",
            Activity::Maintenance => "maintenance",
            Activity::Provisioning => "provisioning",
            Activity::Mail => "mail",
            Activity::LogoutDelivery => "logout_delivery",
            Activity::SsfDelivery => "ssf_delivery",
        }
    }
    pub fn current() -> Self {
        ACTIVITY.get()
    }
}
thread_local! { static ACTIVITY: Cell<Activity> = const { Cell::new(Activity::Foreground) }; }
/// Attributes storage work on this thread to `activity` while `f` runs.
pub fn in_activity<T>(activity: Activity, f: impl FnOnce() -> T) -> T {
    struct Reset(Activity);
    impl Drop for Reset {
        fn drop(&mut self) {
            ACTIVITY.set(self.0);
        }
    }
    let _reset = Reset(ACTIVITY.replace(activity));
    f()
}

/// One histogram per activity.
#[derive(Default)]
pub struct ByActivity([Histogram; 6]);
impl ByActivity {
    pub fn get(&self, activity: Activity) -> &Histogram {
        &self.0[activity as usize]
    }
    fn snapshot(&self) -> Value {
        Value::Object(
            Activity::ALL
                .iter()
                .map(|a| (a.label().to_owned(), self.get(*a).snapshot()))
                .collect(),
        )
    }
}

/// Times one section into a total and a per-activity histogram, optionally
/// counting callers inside it.
pub struct Section<'a> {
    total: &'a Histogram,
    activity: &'a Histogram,
    occupancy: Option<&'a Occupancy>,
    start: Instant,
}
impl<'a> Section<'a> {
    fn new(total: &'a Histogram, by: &'a ByActivity, occupancy: Option<&'a Occupancy>) -> Self {
        if let Some(occupancy) = occupancy {
            occupancy.enter();
        }
        Self {
            total,
            activity: by.get(Activity::current()),
            occupancy,
            start: Instant::now(),
        }
    }
}
impl Drop for Section<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.total.observe(elapsed);
        self.activity.observe(elapsed);
        if let Some(occupancy) = self.occupancy {
            occupancy.leave();
        }
    }
}

/// Where a storage read ran: a read snapshot, under the writer, or an optimistic
/// preparation outside the writer (each uncached preparation read is a short read
/// transaction and, on PostgreSQL, one pool checkout).
#[derive(Clone, Copy)]
pub enum ReadContext {
    Read,
    Writer,
    Prepared,
}
impl ReadContext {
    const ALL: [ReadContext; 3] = [
        ReadContext::Read,
        ReadContext::Writer,
        ReadContext::Prepared,
    ];
    fn label(self) -> &'static str {
        match self {
            ReadContext::Read => "read",
            ReadContext::Writer => "writer",
            ReadContext::Prepared => "prepared",
        }
    }
}

#[derive(Default)]
pub struct Reads {
    points: [AtomicU64; 3],
    bytes: [AtomicU64; 3],
    /// [context][bounded, unbounded]
    scans: [[Sizes; 2]; 3],
}
impl Reads {
    pub fn point(&self, context: ReadContext, bytes: usize) {
        self.points[context as usize].fetch_add(1, Relaxed);
        self.bytes[context as usize].fetch_add(bytes as u64, Relaxed);
    }
    pub fn scan(&self, context: ReadContext, bounded: bool, rows: usize, bytes: usize) {
        self.scans[context as usize][usize::from(!bounded)].observe(rows as u64);
        self.bytes[context as usize].fetch_add(bytes as u64, Relaxed);
    }
    pub fn points(&self, context: ReadContext) -> u64 {
        self.points[context as usize].load(Relaxed)
    }
    pub fn bytes(&self, context: ReadContext) -> u64 {
        self.bytes[context as usize].load(Relaxed)
    }
    pub fn scans(&self, context: ReadContext, bounded: bool) -> &Sizes {
        &self.scans[context as usize][usize::from(!bounded)]
    }
    fn snapshot(&self) -> Value {
        Value::Object(
            ReadContext::ALL
                .iter()
                .map(|c| {
                    (
                        c.label().to_owned(),
                        json!({"point_reads":self.points(*c),"bytes":self.bytes(*c),"bounded_scans":self.scans(*c, true).snapshot(),"unbounded_scans":self.scans(*c, false).snapshot()}),
                    )
                })
                .collect(),
        )
    }
}

/// Why one remote signing attempt failed: one fixed label per failure site. A
/// label never carries a signer, key or domain name, URL, path, status code,
/// response body or error text; an HTTP status is reduced to its class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteSigningFailure {
    StoredKey,
    ConfigurationBinding,
    SignerConfiguration,
    CredentialRead,
    CredentialShape,
    CaSetup,
    ClientSetup,
    Transport,
    Http3xx,
    Http4xx,
    Http5xx,
    HttpOther,
    ResponseSize,
    ResponseShape,
    ResponseVersion,
    SignatureVerification,
    Encoding,
    EditionUnsupported,
}
impl RemoteSigningFailure {
    pub const COUNT: usize = 18;
    pub const ALL: [RemoteSigningFailure; Self::COUNT] = [
        RemoteSigningFailure::StoredKey,
        RemoteSigningFailure::ConfigurationBinding,
        RemoteSigningFailure::SignerConfiguration,
        RemoteSigningFailure::CredentialRead,
        RemoteSigningFailure::CredentialShape,
        RemoteSigningFailure::CaSetup,
        RemoteSigningFailure::ClientSetup,
        RemoteSigningFailure::Transport,
        RemoteSigningFailure::Http3xx,
        RemoteSigningFailure::Http4xx,
        RemoteSigningFailure::Http5xx,
        RemoteSigningFailure::HttpOther,
        RemoteSigningFailure::ResponseSize,
        RemoteSigningFailure::ResponseShape,
        RemoteSigningFailure::ResponseVersion,
        RemoteSigningFailure::SignatureVerification,
        RemoteSigningFailure::Encoding,
        RemoteSigningFailure::EditionUnsupported,
    ];
    pub fn label(self) -> &'static str {
        match self {
            RemoteSigningFailure::StoredKey => "stored_key",
            RemoteSigningFailure::ConfigurationBinding => "configuration_binding",
            RemoteSigningFailure::SignerConfiguration => "signer_configuration",
            RemoteSigningFailure::CredentialRead => "credential_read",
            RemoteSigningFailure::CredentialShape => "credential_shape",
            RemoteSigningFailure::CaSetup => "ca_setup",
            RemoteSigningFailure::ClientSetup => "client_setup",
            RemoteSigningFailure::Transport => "transport",
            RemoteSigningFailure::Http3xx => "http_3xx",
            RemoteSigningFailure::Http4xx => "http_4xx",
            RemoteSigningFailure::Http5xx => "http_5xx",
            RemoteSigningFailure::HttpOther => "http_other",
            RemoteSigningFailure::ResponseSize => "response_size",
            RemoteSigningFailure::ResponseShape => "response_shape",
            RemoteSigningFailure::ResponseVersion => "response_version",
            RemoteSigningFailure::SignatureVerification => "signature_verification",
            RemoteSigningFailure::Encoding => "encoding",
            RemoteSigningFailure::EditionUnsupported => "edition_unsupported",
        }
    }
    /// The class of a non-success HTTP status, never the status itself.
    pub fn status(status: reqwest::StatusCode) -> Self {
        if status.is_redirection() {
            RemoteSigningFailure::Http3xx
        } else if status.is_client_error() {
            RemoteSigningFailure::Http4xx
        } else if status.is_server_error() {
            RemoteSigningFailure::Http5xx
        } else {
            RemoteSigningFailure::HttpOther
        }
    }
}

#[derive(Default)]
pub struct Telemetry {
    pub(crate) background: crate::background::Stats,
    pub(crate) background_executor:
        std::sync::Mutex<std::sync::Weak<crate::background::Background>>,
    pub write_wait: Histogram,
    pub write_hold: Histogram,
    pub pool_wait: Histogram,
    pub signing: Histogram,
    pub password: Histogram,
    pub cleanup: Histogram,
    pub signing_errors: AtomicU64,
    /// Remote signing failures by [`RemoteSigningFailure`]; each is also in `signing_errors`.
    pub remote_signing_failures: [AtomicU64; RemoteSigningFailure::COUNT],
    pub cleanup_errors: AtomicU64,
    pub alert_delivery_errors: AtomicU64,
    pub scanned_records: AtomicU64,
    pub optimistic_conflicts: AtomicU64,
    /// Writer contention: callers waiting for the writer, and wait/hold by activity.
    pub write_waiters: Occupancy,
    /// Durable commit time, part of the writer hold.
    pub commit: Histogram,
    pub activity_write_wait: ByActivity,
    pub activity_write_hold: ByActivity,
    /// PostgreSQL pool: callers inside checkout, connections checked out, how long
    /// they stay out, and how long new connections take to establish.
    pub pool_waiters: Occupancy,
    pub pool_in_use: Occupancy,
    pub pool_capacity: AtomicU64,
    pub pool_hold: Histogram,
    pub activity_pool_wait: ByActivity,
    pub activity_pool_hold: ByActivity,
    pub pool_connect: Histogram,
    pub pool_connect_errors: AtomicU64,
    pub pool_timeouts: AtomicU64,
    pub pool_discarded: AtomicU64,
    /// Rows and bytes materialized, by where they were read.
    pub reads: Reads,
    pub snapshot_records: AtomicU64,
    /// Optimistic preparation: callback time outside the writer, validation time
    /// under it, and why attempts did not commit.
    pub prepared_attempts: AtomicU64,
    pub prepared_prepare: Histogram,
    pub prepared_validate: Histogram,
    pub prepared_validated_records: AtomicU64,
    pub prepared_expired: AtomicU64,
    pub prepared_exhausted: AtomicU64,
}
impl Telemetry {
    pub(crate) fn writer_wait(&self) -> Section<'_> {
        Section::new(
            &self.write_wait,
            &self.activity_write_wait,
            Some(&self.write_waiters),
        )
    }
    pub(crate) fn writer_hold(&self) -> Section<'_> {
        Section::new(&self.write_hold, &self.activity_write_hold, None)
    }
    pub(crate) fn pool_checkout(&self) -> Section<'_> {
        Section::new(
            &self.pool_wait,
            &self.activity_pool_wait,
            Some(&self.pool_waiters),
        )
    }
    /// Counts one remote signing failure and logs only its fixed label. The event
    /// has no parent span, so request fields are not attached to it.
    pub(crate) fn remote_signing_failed(&self, reason: RemoteSigningFailure) {
        self.remote_signing_failures[reason as usize].fetch_add(1, Relaxed);
        tracing::warn!(parent: None, reason = reason.label(), "remote signing failed");
    }
    fn remote_signing_snapshot(&self) -> Value {
        Value::Object(
            RemoteSigningFailure::ALL
                .iter()
                .map(|reason| {
                    (
                        reason.label().to_owned(),
                        json!(self.remote_signing_failures[*reason as usize].load(Relaxed)),
                    )
                })
                .collect(),
        )
    }
    pub(crate) fn pool_returned(&self, activity: Activity, held: Duration) {
        self.pool_hold.observe(held);
        self.activity_pool_hold.get(activity).observe(held);
        self.pool_in_use.leave();
    }
    pub fn snapshot(&self) -> Value {
        json!({"write_wait":self.write_wait.snapshot(),"write_hold":self.write_hold.snapshot(),"pool_wait":self.pool_wait.snapshot(),"signing":self.signing.snapshot(),"password":self.password.snapshot(),"cleanup":self.cleanup.snapshot(),"signing_errors":self.signing_errors.load(Relaxed),"remote_signing_failures":self.remote_signing_snapshot(),"cleanup_errors":self.cleanup_errors.load(Relaxed),"alert_delivery_errors":self.alert_delivery_errors.load(Relaxed),"scanned_records":self.scanned_records.load(Relaxed),"optimistic_conflicts":self.optimistic_conflicts.load(Relaxed),
            "background":self.background.snapshot(),
            "writer":{"waiters":self.write_waiters.snapshot(),"commit":self.commit.snapshot(),"wait_by_activity":self.activity_write_wait.snapshot(),"hold_by_activity":self.activity_write_hold.snapshot()},
            "pool":{"capacity":self.pool_capacity.load(Relaxed),"waiters":self.pool_waiters.snapshot(),"in_use":self.pool_in_use.snapshot(),"hold":self.pool_hold.snapshot(),"wait_by_activity":self.activity_pool_wait.snapshot(),"hold_by_activity":self.activity_pool_hold.snapshot(),"connect":self.pool_connect.snapshot(),"connect_errors":self.pool_connect_errors.load(Relaxed),"timeouts":self.pool_timeouts.load(Relaxed),"discarded":self.pool_discarded.load(Relaxed)},
            "reads":self.reads.snapshot(),"snapshot_records":self.snapshot_records.load(Relaxed),
            "prepared":{"attempts":self.prepared_attempts.load(Relaxed),"prepare":self.prepared_prepare.snapshot(),"validate":self.prepared_validate.snapshot(),"validated_records":self.prepared_validated_records.load(Relaxed),"expired":self.prepared_expired.load(Relaxed),"exhausted":self.prepared_exhausted.load(Relaxed)}})
    }
    pub fn render(&self, output: &mut String) {
        self.background.render(output);
        for (name, histogram) in [
            ("storage_write_wait", &self.write_wait),
            ("storage_write_hold", &self.write_hold),
            ("storage_pool_wait", &self.pool_wait),
            ("signing", &self.signing),
            ("password_verification", &self.password),
            ("cleanup", &self.cleanup),
            ("storage_commit", &self.commit),
            ("storage_pool_hold", &self.pool_hold),
            ("storage_pool_connect", &self.pool_connect),
            ("prepared_prepare", &self.prepared_prepare),
            ("prepared_validate", &self.prepared_validate),
        ] {
            writeln!(output, "# TYPE riauth_{name}_seconds histogram").unwrap();
            histogram.render(output, &format!("riauth_{name}_seconds"), "");
        }
        for (name, by) in [
            ("storage_activity_write_wait", &self.activity_write_wait),
            ("storage_activity_write_hold", &self.activity_write_hold),
            ("storage_activity_pool_wait", &self.activity_pool_wait),
            ("storage_activity_pool_hold", &self.activity_pool_hold),
        ] {
            writeln!(output, "# TYPE riauth_{name}_seconds histogram").unwrap();
            // Like route labels, an activity's series appears once it has been observed.
            for activity in Activity::ALL.into_iter().filter(|a| by.get(*a).count() > 0) {
                by.get(activity).render(
                    output,
                    &format!("riauth_{name}_seconds"),
                    &format!("activity=\"{}\"", activity.label()),
                );
            }
        }
        writeln!(output, "# TYPE riauth_storage_scan_rows histogram").unwrap();
        for context in ReadContext::ALL {
            for (bounded, limit) in [(true, "bounded"), (false, "unbounded")] {
                if self.reads.scans(context, bounded).count() == 0 {
                    continue;
                }
                self.reads.scans(context, bounded).render(
                    output,
                    "riauth_storage_scan_rows",
                    &format!("context=\"{}\",limit=\"{limit}\"", context.label()),
                );
            }
        }
        for (name, values) in [
            ("storage_point_reads", &self.reads.points),
            ("storage_read_bytes", &self.reads.bytes),
        ] {
            writeln!(output, "# TYPE riauth_{name}_total counter").unwrap();
            for context in ReadContext::ALL {
                writeln!(
                    output,
                    "riauth_{name}_total{{context=\"{}\"}} {}",
                    context.label(),
                    values[context as usize].load(Relaxed)
                )
                .unwrap();
            }
        }
        for (name, counter) in [
            ("signing_errors", &self.signing_errors),
            ("cleanup_errors", &self.cleanup_errors),
            ("alert_delivery_errors", &self.alert_delivery_errors),
            ("storage_scanned_records", &self.scanned_records),
            ("storage_optimistic_conflicts", &self.optimistic_conflicts),
            ("storage_snapshot_records", &self.snapshot_records),
            ("storage_pool_connect_errors", &self.pool_connect_errors),
            ("storage_pool_timeouts", &self.pool_timeouts),
            ("storage_pool_discarded", &self.pool_discarded),
            ("prepared_attempts", &self.prepared_attempts),
            (
                "prepared_validated_records",
                &self.prepared_validated_records,
            ),
            ("prepared_expired", &self.prepared_expired),
            ("prepared_exhausted", &self.prepared_exhausted),
        ] {
            writeln!(
                output,
                "# TYPE riauth_{name}_total counter\nriauth_{name}_total {}",
                counter.load(Relaxed)
            )
            .unwrap();
        }
        writeln!(
            output,
            "# TYPE riauth_remote_signing_failures_total counter"
        )
        .unwrap();
        // Like activity labels, a reason's series appears once it has been observed.
        for reason in RemoteSigningFailure::ALL {
            let count = self.remote_signing_failures[reason as usize].load(Relaxed);
            if count > 0 {
                writeln!(
                    output,
                    "riauth_remote_signing_failures_total{{reason=\"{}\"}} {count}",
                    reason.label()
                )
                .unwrap();
            }
        }
        for (name, occupancy) in [
            ("storage_write_waiters", &self.write_waiters),
            ("storage_pool_waiters", &self.pool_waiters),
            ("storage_pool_in_use", &self.pool_in_use),
        ] {
            writeln!(
                output,
                "# TYPE riauth_{name} gauge\nriauth_{name} {}\n# TYPE riauth_{name}_peak gauge\nriauth_{name}_peak {}",
                occupancy.current(),
                occupancy.peak()
            )
            .unwrap();
        }
        writeln!(
            output,
            "# TYPE riauth_storage_pool_capacity gauge\nriauth_storage_pool_capacity {}",
            self.pool_capacity.load(Relaxed)
        )
        .unwrap();
    }
}
