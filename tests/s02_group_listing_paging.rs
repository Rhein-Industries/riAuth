//! One local encrypted-redb comparison. No service, clock override or product hook.
//! Existing storage histograms measure raw rows retained by each fetch, not heap
//! bytes, peak decoded/output memory or RSS. Timers measure actual list calls.
//! All output is fixed labels and numeric data; no allocation policy is changed.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use riauth::{
    agent::{Agent, NewAgent, Permission},
    config::{self, Config},
    core::Core,
    crypto,
    error::{Error, Result},
    model::{Group, NewUser, User, UserPatch, UserView},
    store::{Tx, maintenance::PAGE},
    telemetry::{ReadContext, Sizes},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    hint::black_box,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tempfile::TempDir;
use zeroize::Zeroizing;

const PASSWORD: &str = "s02-synthetic-fixture-password-only";
const USERS: usize = 128;
const GROUPS: usize = 513;
const SELECTED: [usize; 3] = [0, 256, 512];
const WARMUPS: usize = 5;
const SAMPLES: usize = 10;
const FIXTURE_LIMIT: Duration = Duration::from_secs(300);
const THREAD_LIMIT: Duration = Duration::from_secs(10);
// Pinned public Sizes::render schema at fab6721; a changed schema refuses.
const SCAN_BOUNDS: [u64; 7] = [0, 1, 16, 128, 1_024, 8_192, 65_536];

#[derive(Clone, Copy, PartialEq, Eq)]
struct ScanHistogram {
    buckets: [u64; 8], // cumulative finite bounds above, followed by +Inf
    count: u64,
    rows: u64,
}
impl ScanHistogram {
    fn parse(text: &str) -> Option<Self> {
        if text.len() > 4096 || text.lines().count() != 10 {
            return None;
        }
        let mut buckets = [None; 8];
        let mut count = None;
        let mut rows = None;
        for line in text.lines() {
            let (name, value) = line.rsplit_once(' ')?;
            let number: u64 = value.parse().ok()?;
            if number.to_string() != value {
                return None;
            }
            let slot = if name == "s02_scan_rows_count" {
                &mut count
            } else if name == "s02_scan_rows_sum" {
                &mut rows
            } else {
                let index = (0..8).find(|index| {
                    let bound = if *index == 7 {
                        "+Inf".to_owned()
                    } else {
                        SCAN_BOUNDS[*index].to_string()
                    };
                    name == format!("s02_scan_rows_bucket{{le=\"{bound}\"}}")
                })?;
                &mut buckets[index]
            };
            if slot.replace(number).is_some() {
                return None;
            }
        }
        let buckets: [u64; 8] = buckets
            .into_iter()
            .collect::<Option<Vec<_>>>()?
            .try_into()
            .ok()?;
        let count = count?;
        let rows = rows?;
        if buckets[7] != count || buckets.windows(2).any(|pair| pair[0] > pair[1]) {
            return None;
        }
        Some(Self {
            buckets,
            count,
            rows,
        })
    }
    fn read(sizes: &Sizes) -> Self {
        let mut text = String::new();
        sizes.render(&mut text, "s02_scan_rows", "");
        let value = Self::parse(&text).expect("native scan histogram schema");
        assert!(
            value.count == sizes.count() && value.rows == sizes.sum(),
            "native scan histogram counter consistency"
        );
        value
    }
    fn delta(self, before: Self) -> Self {
        Self {
            buckets: std::array::from_fn(|index| {
                self.buckets[index]
                    .checked_sub(before.buckets[index])
                    .expect("native scan bucket went backwards")
            }),
            count: self
                .count
                .checked_sub(before.count)
                .expect("native scan count went backwards"),
            rows: self
                .rows
                .checked_sub(before.rows)
                .expect("native scan rows went backwards"),
        }
    }
    fn expected(sizes: &[u64]) -> Self {
        Self {
            buckets: std::array::from_fn(|index| {
                sizes
                    .iter()
                    .filter(|size| index == 7 || **size <= SCAN_BOUNDS[index])
                    .count() as u64
            }),
            count: sizes.len() as u64,
            rows: sizes.iter().sum(),
        }
    }
    fn over_128(self) -> u64 {
        self.count
            .checked_sub(self.buckets[3])
            .expect("native 128 bucket exceeds count")
    }
}
fn telemetry_self_check() {
    // Exercise the actual safe public observer, independent literal expected
    // frequencies, and strict parser refusals; no candidate list code is run.
    let sizes = Sizes::default();
    assert!(
        ScanHistogram::read(&sizes) == ScanHistogram::expected(&[]),
        "empty native histogram"
    );
    for size in [128, 128, 128, 128, 1] {
        sizes.observe(size);
    }
    let actual = ScanHistogram::read(&sizes);
    assert!(
        actual
            == ScanHistogram {
                buckets: [0, 1, 1, 5, 5, 5, 5, 5],
                count: 5,
                rows: 513,
            },
        "literal native histogram self-check"
    );
    let mut text = String::new();
    sizes.render(&mut text, "s02_scan_rows", "");
    let duplicate = text.replacen(
        "s02_scan_rows_bucket{le=\"0\"} 0",
        "s02_scan_rows_bucket{le=\"1\"} 0",
        1,
    );
    let malformed = text.replacen("s02_scan_rows_sum 513", "s02_scan_rows_sum false", 1);
    let noncanonical = text.replacen("s02_scan_rows_sum 513", "s02_scan_rows_sum 0513", 1);
    let inconsistent = text.replacen("s02_scan_rows_count 5", "s02_scan_rows_count 6", 1);
    let missing = text.replacen("s02_scan_rows_count 5\n", "", 1);
    assert!(
        [duplicate, malformed, noncanonical, inconsistent, missing]
            .iter()
            .all(|case| ScanHistogram::parse(case).is_none()),
        "native histogram parser refusal"
    );
    println!(
        "{}",
        json!({"schema":"riauth.s02-telemetry-self-check/v1","scan_count":5,"rows":513,"buckets":[0,1,1,5,5,5,5,5],"parser_refusal_cases":5,"heap_claim":false,"rss_claim":false})
    );
}

// Whole frozen method is byte-identical to cdebc/d644/fab6721 list_groups.
// Deref supplies Core's actual Store and public principal; no frozen authority
// helper or copied scan implementation can replace the live shared service.
struct Frozen<'a>(&'a Core);
impl Deref for Frozen<'_> {
    type Target = Core;
    fn deref(&self) -> &Core {
        self.0
    }
}
impl Frozen<'_> {
    pub fn list_groups(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<Group>("groups")?
                    .into_iter()
                    .filter(|(_, u)| actor.allows("group.read", &format!("group/{}", u.name)))
                    .map(|(_, g)| g)
                    .collect::<Vec<_>>()
            ))
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
    Control,
    Paged,
}
impl Lane {
    fn label(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Paged => "paged",
        }
    }
    fn list(self, core: &Core, token: &str) -> Result<Value> {
        match self {
            Self::Control => Frozen(core).list_groups(token),
            Self::Paged => core.list_groups(token),
        }
    }
}

#[derive(Clone, Copy)]
struct Budget(Instant);
impl Budget {
    fn check(self) {
        assert!(Instant::now() < self.0, "s02 fixture deadline");
    }
    fn thread_wait(self) -> Duration {
        self.check();
        self.0
            .saturating_duration_since(Instant::now())
            .min(THREAD_LIMIT)
    }
}

// Cooperative checks alone cannot interrupt stalled filesystem IO or join.
// The parked owned watchdog requests non-coredumping process exit124 at 300s.
// A future outer supervisor must enforce its deadline even if exit/IO stalls,
// retain that failure and clean only its private fixture root.
// Normal success/unwind cancels and joins this thread. No product hook/clock.
struct Watchdog {
    stop: Arc<(Mutex<bool>, Condvar)>,
    handle: Option<JoinHandle<()>>,
}
impl Watchdog {
    fn new(budget: Budget) -> Self {
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let state = stop.clone();
        let (ready, receiver) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            let (lock, wake) = &*state;
            let mut stopped = lock.lock().unwrap_or_else(|_| std::process::exit(124));
            if ready.send(()).is_err() {
                return;
            }
            while !*stopped {
                let remaining = budget.0.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    std::process::exit(124);
                }
                let (next, _) = wake
                    .wait_timeout(stopped, remaining)
                    .unwrap_or_else(|_| std::process::exit(124));
                stopped = next;
            }
        });
        let owned = Self {
            stop,
            handle: Some(handle),
        };
        require(
            receiver.recv_timeout(budget.thread_wait()),
            "watchdog startup",
        );
        // Acquiring after the rendezvous proves watchdog released the mutex
        // into its wait; thread/TLS startup is outside every measured window.
        drop(
            owned
                .stop
                .0
                .lock()
                .unwrap_or_else(|_| std::process::exit(124)),
        );
        owned
    }
}
impl Drop for Watchdog {
    fn drop(&mut self) {
        *self
            .stop
            .0
            .lock()
            .unwrap_or_else(|_| std::process::exit(124)) = true;
        self.stop.1.notify_one();
        if let Some(handle) = self.handle.take() {
            if handle.join().is_err() {
                std::process::exit(124);
            }
        }
    }
}

struct OwnedWorker<T> {
    handle: Option<JoinHandle<T>>,
}
impl<T> OwnedWorker<T> {
    fn new(work: impl FnOnce() -> T + Send + 'static) -> Self
    where
        T: Send + 'static,
    {
        Self {
            handle: Some(thread::spawn(work)),
        }
    }
    fn join(mut self) -> T {
        require(
            self.handle.take().expect("owned worker").join(),
            "owned worker panic",
        )
    }
}
impl<T> Drop for OwnedWorker<T> {
    fn drop(&mut self) {
        // Workers have bounded channel waits. If IO never completes, the
        // process deadline above bounds this join; never silently detach.
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn require<T, E>(result: std::result::Result<T, E>, label: &'static str) -> T {
    result.unwrap_or_else(|_| panic!("{label}"))
}
fn core_ok<T>(result: Result<T>, label: &'static str) -> T {
    result.unwrap_or_else(|error| {
        panic!(
            "{label}: status={} code={}",
            error.status.as_u16(),
            error.code
        )
    })
}
type Snapshot = BTreeMap<String, Value>;
fn snapshot(core: &Core) -> Snapshot {
    core_ok(core.store.read(|tx| tx.snapshot()), "full snapshot")
}
fn same_snapshot(core: &Core, expected: &Snapshot) {
    assert!(snapshot(core) == *expected, "full durable snapshot changed");
}
fn token(value: &Value) -> Zeroizing<String> {
    Zeroizing::new(
        value["credential"]["token"]
            .as_str()
            .expect("private credential response")
            .to_owned(),
    )
}
fn user_id(index: usize) -> String {
    format!("s02-user-{index:03}")
}
fn user_name(index: usize) -> String {
    format!("s02-person-{index:03}")
}
fn group_name(index: usize) -> String {
    format!("s02-group-{index:03}")
}
fn expected(count: usize, scoped: bool, removed: Option<usize>) -> Value {
    // Construct from fixture constants, not observed output or the frozen code.
    Value::Array(
        (0..count)
            .filter(|index| !scoped || SELECTED.contains(index))
            .map(|index| {
                let members: Vec<_> = (0..USERS)
                    .filter(|member| !(removed == Some(index) && *member == 0))
                    .map(user_id)
                    .collect();
                json!({"name":group_name(index),"members":members})
            })
            .collect(),
    )
}

struct Credentials {
    admin: Zeroizing<String>,
    scoped: Zeroizing<String>,
    empty: Zeroizing<String>,
}
impl Credentials {
    fn copy(&self) -> Self {
        Self {
            admin: self.admin.clone(),
            scoped: self.scoped.clone(),
            empty: self.empty.clone(),
        }
    }
}
struct Fixture {
    core: Core, // close Store before deleting owned directory
    _dir: TempDir,
    credentials: Credentials,
}
struct Seed {
    _dir: TempDir,
    config: Config,
    credentials: Credentials,
    state: Snapshot,
}
fn close_dir(directory: TempDir) {
    let path = directory.path().to_path_buf();
    require(directory.close(), "owned fixture directory cleanup");
    assert!(
        !require(path.try_exists(), "owned fixture cleanup postcondition"),
        "owned fixture directory remains"
    );
}
impl Fixture {
    fn close(self) {
        let Self {
            core,
            _dir,
            credentials,
        } = self;
        drop(core);
        drop(credentials);
        close_dir(_dir);
    }
}
fn private_dir(root: &Path) -> TempDir {
    require(
        tempfile::Builder::new()
            .prefix("s02-group-list-")
            .tempdir_in(root),
        "owned private fixture directory",
    )
}
fn fixture_root() -> PathBuf {
    let selected = require(
        std::env::var_os("CARGO_TARGET_DIR").ok_or(()),
        "private target required",
    );
    let root = require(
        PathBuf::from(selected).canonicalize(),
        "private target identity",
    );
    let own = require(
        require(std::env::current_dir(), "worktree identity")
            .join("target")
            .canonicalize(),
        "own target identity",
    );
    assert!(root == own, "refuse target outside own worktree");
    root
}
impl Seed {
    fn new(root: &Path, budget: Budget) -> Self {
        budget.check();
        let dir = private_dir(root);
        let key = dir.path().join("storage.key");
        require(
            config::write_private(&key, crypto::random_token("").as_bytes(), false),
            "private encryption key",
        );
        let config = Config {
            data_dir: dir.path().join("data"),
            database_key_file: Some(key),
            ..Default::default()
        };
        let core = core_ok(
            Core::initialize(
                config.clone(),
                NewUser {
                    username: "admin".into(),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: "S02 synthetic administrator".into(),
                    admin: true,
                },
            ),
            "encrypted seed initialize",
        );
        let login = core_ok(
            core.login("admin".into(), PASSWORD.into(), None),
            "seed login",
        );
        let admin: Zeroizing<String> = Zeroizing::new(
            login["session_token"]
                .as_str()
                .expect("private login response")
                .into(),
        );
        let template: User = core_ok(
            core.store.read(|tx| {
                let id = tx
                    .get::<String>("usernames", "admin")?
                    .ok_or_else(Error::unauthorized)?;
                tx.get("users", &id)?.ok_or_else(Error::unauthorized)
            }),
            "seed User template",
        );
        core_ok(
            core.store.write(|tx| {
                for index in 0..USERS {
                    budget.check();
                    let mut user = template.clone();
                    user.id = user_id(index);
                    user.username = user_name(index);
                    user.admin = false;
                    user.display_name = user.username.clone();
                    tx.put("users", &user.id, &user)?;
                    tx.put("usernames", &user.username, &user.id)?;
                }
                Ok(())
            }),
            "synthetic stable Users",
        );
        let scoped = token(&core_ok(
            core.create_agent(
                &admin,
                NewAgent {
                    id: "s02-scoped".into(),
                    ttl: 3600,
                    parent: None,
                    permissions: SELECTED
                        .iter()
                        .map(|index| Permission {
                            action: "group.read".into(),
                            resource: format!("group/{}", group_name(*index)),
                        })
                        .collect(),
                },
            ),
            "scoped public credential creation",
        ));
        let empty = token(&core_ok(
            core.create_agent(
                &admin,
                NewAgent {
                    id: "s02-empty".into(),
                    ttl: 3600,
                    parent: None,
                    permissions: vec![Permission {
                        action: "group.read".into(),
                        resource: "group/s02-not-present".into(),
                    }],
                },
            ),
            "empty-scope public credential creation",
        ));
        // Scoped credentials may add accepted edition-provenance observations
        // on the next normal open. Settle that PUBLIC startup once before the
        // closed seed snapshot; copied lanes must match it with no exclusions.
        drop(core);
        let core = core_ok(Core::open(config.clone()), "seed public startup");
        let state = snapshot(&core);
        drop(core); // closed file is the only seed copy source
        Self {
            _dir: dir,
            config,
            credentials: Credentials {
                admin,
                scoped,
                empty,
            },
            state,
        }
    }
    fn open(&self, root: &Path, budget: Budget) -> Fixture {
        budget.check();
        let dir = private_dir(root);
        let mut config = self.config.clone();
        config.data_dir = dir.path().join("data");
        require(
            config::private_dir(&config.data_dir),
            "private copied data directory",
        );
        require(
            std::fs::copy(
                self.config.data_dir.join("riauth.redb"),
                config.data_dir.join("riauth.redb"),
            ),
            "closed seed copy",
        );
        let core = core_ok(Core::open(config), "encrypted copied fixture open");
        same_snapshot(&core, &self.state);
        let mut actual = core.config.clone();
        actual.data_dir = self.config.data_dir.clone();
        assert!(
            require(serde_json::to_value(actual), "copied configuration")
                == require(serde_json::to_value(&self.config), "seed configuration"),
            "normalized configuration changed"
        );
        Fixture {
            core,
            _dir: dir,
            credentials: self.credentials.copy(),
        }
    }
    fn close(fixture: Fixture) -> Self {
        let Fixture {
            core,
            _dir,
            credentials,
        } = fixture;
        let config = core.config.clone();
        let state = snapshot(&core);
        drop(core);
        Self {
            _dir,
            config,
            credentials,
            state,
        }
    }
    fn cleanup(self) {
        let Self {
            _dir,
            config,
            credentials,
            state,
        } = self;
        drop(state);
        drop(credentials);
        drop(config);
        close_dir(_dir);
    }
}

fn populate(core: &Core, count: usize, budget: Budget) {
    core_ok(
        core.store.write(|tx| {
            let members: BTreeSet<_> = (0..USERS).map(user_id).collect();
            for index in 0..count {
                budget.check();
                let name = group_name(index);
                tx.put(
                    "groups",
                    &name,
                    &Group {
                        name: name.clone(),
                        members: members.clone(),
                    },
                )?;
            }
            Ok(())
        }),
        "synthetic canonical Groups",
    );
}

#[derive(Clone, Copy)]
struct Counters {
    points: u64,
    bytes: u64,
    bounded: u64,
    bounded_rows: u64,
    unbounded: u64,
    unbounded_rows: u64,
    bounded_histogram: ScanHistogram,
    unbounded_histogram: ScanHistogram,
    writer_holds: u64,
    commits: u64,
}
impl Counters {
    fn read(core: &Core) -> Self {
        let telemetry = core.store.telemetry();
        let reads = &telemetry.reads;
        let bounded_histogram = ScanHistogram::read(reads.scans(ReadContext::Read, true));
        let unbounded_histogram = ScanHistogram::read(reads.scans(ReadContext::Read, false));
        Self {
            points: reads.points(ReadContext::Read),
            bytes: reads.bytes(ReadContext::Read),
            bounded: bounded_histogram.count,
            bounded_rows: bounded_histogram.rows,
            unbounded: unbounded_histogram.count,
            unbounded_rows: unbounded_histogram.rows,
            bounded_histogram,
            unbounded_histogram,
            writer_holds: telemetry.write_hold.count(),
            commits: telemetry.commit.count(),
        }
    }
    fn delta(self, before: Self) -> Self {
        let subtract = |after: u64, before: u64| {
            after
                .checked_sub(before)
                .expect("native counter went backwards")
        };
        Self {
            points: subtract(self.points, before.points),
            bytes: subtract(self.bytes, before.bytes),
            bounded: subtract(self.bounded, before.bounded),
            bounded_rows: subtract(self.bounded_rows, before.bounded_rows),
            unbounded: subtract(self.unbounded, before.unbounded),
            unbounded_rows: subtract(self.unbounded_rows, before.unbounded_rows),
            bounded_histogram: self.bounded_histogram.delta(before.bounded_histogram),
            unbounded_histogram: self.unbounded_histogram.delta(before.unbounded_histogram),
            writer_holds: subtract(self.writer_holds, before.writer_holds),
            commits: subtract(self.commits, before.commits),
        }
    }
    fn assert_read(self, lane: Lane, count: usize) {
        assert!(
            self.writer_holds == 0 && self.commits == 0,
            "reader acquired writer or committed"
        );
        let empty = ScanHistogram::expected(&[]);
        match lane {
            Lane::Control => {
                assert!(
                    (
                        self.bounded,
                        self.bounded_rows,
                        self.unbounded,
                        self.unbounded_rows
                    ) == (0, 0, 1, count as u64),
                    "control complete scan accounting"
                );
                assert!(
                    self.bounded_histogram == empty
                        && self.unbounded_histogram == ScanHistogram::expected(&[count as u64]),
                    "control exact native row histogram"
                );
            }
            Lane::Paged => {
                assert!(
                    (
                        self.bounded,
                        self.bounded_rows,
                        self.unbounded,
                        self.unbounded_rows
                    ) == ((count / PAGE + 1) as u64, count as u64, 0, 0),
                    "paged exact native scan accounting"
                );
                let mut sizes = vec![PAGE as u64; count / PAGE];
                sizes.push((count % PAGE) as u64);
                assert!(
                    self.unbounded_histogram == empty
                        && self.bounded_histogram == ScanHistogram::expected(&sizes),
                    "paged exact native row histogram"
                );
            }
        }
    }
    fn fetches_over_128(self) -> u64 {
        self.bounded_histogram.over_128() + self.unbounded_histogram.over_128()
    }
}

fn checked_read(
    core: &Core,
    lane: Lane,
    credential: &str,
    oracle: &Value,
    count: usize,
    budget: Budget,
) -> Counters {
    budget.check();
    let state = snapshot(core);
    let before = Counters::read(core);
    let value = core_ok(lane.list(core, credential), "list result");
    let delta = Counters::read(core).delta(before);
    assert!(value == *oracle, "complete independent Group JSON oracle");
    delta.assert_read(lane, count);
    same_snapshot(core, &state);
    budget.check();
    delta
}

fn refused_read(core: &Core, lane: Lane, credential: &str, budget: Budget) {
    budget.check();
    let state = snapshot(core);
    let before = Counters::read(core);
    let error = match lane.list(core, credential) {
        Err(error) => error,
        Ok(_) => panic!("retired or invalid credential accepted"),
    };
    assert!(
        error.status.as_u16() == 401 && error.code == "invalid_token",
        "exact authority refusal"
    );
    let delta = Counters::read(core).delta(before);
    assert!(
        delta.bounded == 0 && delta.unbounded == 0,
        "authority refusal traversed Groups"
    );
    assert!(
        delta.writer_holds == 0 && delta.commits == 0,
        "authority refusal mutated"
    );
    same_snapshot(core, &state);
    budget.check();
}

fn denied_member_write(fixture: &Fixture, budget: Budget) {
    budget.check();
    let before = snapshot(&fixture.core);
    let error = match fixture.core.group_member(
        &fixture.credentials.scoped,
        &group_name(512),
        &user_name(0),
        false,
    ) {
        Err(error) => error,
        Ok(_) => panic!("read-only principal mutated membership"),
    };
    assert!(
        error.status.as_u16() == 403 && error.code == "access_denied",
        "exact Group writer scope refusal"
    );
    same_snapshot(&fixture.core, &before);
    budget.check();
}

fn malformed_write_refusal(fixture: &Fixture, budget: Budget) {
    budget.check();
    let before = snapshot(&fixture.core);
    let result = fixture.core.store.write(|tx| {
        tx.put(
            "groups",
            "s02-malformed",
            &json!({"name":"s02-malformed","members":17}),
        )
    });
    let error = match result {
        Err(error) => error,
        Ok(()) => panic!("malformed Group was accepted"),
    };
    assert!(
        error.status.as_u16() == 500 && error.code == "server_error",
        "canonical Group decode refusal"
    );
    same_snapshot(&fixture.core, &before);
    // No raw import or malformed persisted row is used to create this case.
    // The shared writer rejects it; scan decode errors still propagate in
    // both source bodies, rather than an invented corruption-runtime claim.
    budget.check();
}

#[derive(Clone, Copy)]
struct Observation {
    pair: usize,
    lane: Lane,
    scoped: bool,
    sample: usize,
    elapsed_ns: u64,
    native: Counters,
}
fn emit(observation: Observation) {
    let Observation {
        pair,
        lane,
        scoped,
        sample,
        elapsed_ns,
        native,
    } = observation;
    println!(
        "{}",
        json!({
            "schema":"riauth.s02-group-list-sample/v2", "pair":pair,
            "lane":lane.label(), "scope":if scoped {"scoped"} else {"admin"},
            "sample":sample,"elapsed_ns":elapsed_ns,
            "point_reads":native.points,"native_point_plus_scan_value_bytes":native.bytes,
            "bounded_scans":native.bounded,"bounded_rows":native.bounded_rows,
            "unbounded_scans":native.unbounded,"unbounded_rows":native.unbounded_rows,
            "row_histogram_bounds":[0,1,16,128,1024,8192,65536],
            "bounded_cumulative_buckets":native.bounded_histogram.buckets,
            "unbounded_cumulative_buckets":native.unbounded_histogram.buckets,
            "fetches_over_128_rows":native.fetches_over_128(),
            "writer_holds":native.writer_holds,"commits":native.commits,
            "encrypted_at_rest":true,"latency_interval":"complete_list_call",
            "heap_claim":false,"rss_claim":false
        })
    );
}

fn measure(
    fixture: &Fixture,
    lane: Lane,
    pair: usize,
    scoped: bool,
    observations: &mut Vec<Observation>,
    budget: Budget,
) {
    let credential = if scoped {
        &fixture.credentials.scoped
    } else {
        &fixture.credentials.admin
    };
    let oracle = expected(GROUPS, scoped, None);
    let state = snapshot(&fixture.core);
    for _ in 0..WARMUPS {
        checked_read(&fixture.core, lane, credential, &oracle, GROUPS, budget);
    }
    for sample in 0..SAMPLES {
        budget.check();
        let before = Counters::read(&fixture.core);
        let start = Instant::now();
        let result = black_box(lane.list(black_box(&fixture.core), black_box(credential)));
        let elapsed = start.elapsed();
        // Returned complete JSON stays live. Observer rendering, grading,
        // snapshots and returned-value drop are outside the call timer.
        let native = Counters::read(&fixture.core).delta(before);
        let observation = Observation {
            pair,
            lane,
            scoped,
            sample,
            elapsed_ns: require(u64::try_from(elapsed.as_nanos()), "elapsed numeric bound"),
            native,
        };
        emit(observation); // finite raw numeric outcome before evidence grading
        observations.push(observation);
        let value = core_ok(result, "measured list result");
        assert!(value == oracle, "measured complete independent JSON oracle");
        native.assert_read(lane, GROUPS);
        same_snapshot(&fixture.core, &state);
        drop(value);
        budget.check();
    }
    same_snapshot(&fixture.core, &state);
}

fn percentile(mut values: Vec<u64>, numerator: usize, denominator: usize) -> u64 {
    assert!(!values.is_empty(), "empty measurement summary");
    values.sort_unstable();
    let rank = (values.len() * numerator).div_ceil(denominator);
    values[rank.saturating_sub(1)]
}
fn summaries(observations: &[Observation]) {
    let mut fetch_materialization_improved = true;
    let mut native_equivalent = true;
    let mut scoped_latency_p50_lower_in_every_pair = true;
    for pair in 0..3 {
        for scoped in [true, false] {
            let select = |lane| {
                observations
                    .iter()
                    .filter(move |row| row.pair == pair && row.scoped == scoped && row.lane == lane)
            };
            for lane in [Lane::Control, Lane::Paged] {
                let selected: Vec<_> = select(lane).collect();
                assert!(selected.len() == SAMPLES, "complete sample count");
                println!(
                    "{}",
                    json!({
                        "schema":"riauth.s02-group-list-summary/v2","pair":pair,"lane":lane.label(),
                        "scope":if scoped {"scoped"} else {"admin"},"samples":selected.len(),
                        "median_fetches_over_128_rows":percentile(selected.iter().map(|row| row.native.fetches_over_128()).collect(),1,2),
                        "p50_complete_call_elapsed_ns":percentile(selected.iter().map(|row| row.elapsed_ns).collect(),1,2),
                        "p95_complete_call_elapsed_ns":percentile(selected.iter().map(|row| row.elapsed_ns).collect(),95,100),
                        "heap_claim":false,"rss_claim":false
                    })
                );
            }
            let control_native = select(Lane::Control).next().expect("control sample").native;
            for row in select(Lane::Paged).chain(select(Lane::Control)) {
                native_equivalent &= row.native.points == control_native.points
                    && row.native.bytes == control_native.bytes;
                // These are actual native frequency observations, not a
                // PAGE-based prediction or a memory/throughput inference.
                fetch_materialization_improved &= match row.lane {
                    Lane::Control => {
                        row.native.fetches_over_128() == 1
                            && row.native.unbounded == 1
                            && row.native.unbounded_rows == 513
                    }
                    Lane::Paged => {
                        row.native.fetches_over_128() == 0
                            && row.native.bounded == 5
                            && row.native.bounded_rows == 513
                    }
                };
            }
            if scoped {
                let p50 = |lane| percentile(select(lane).map(|row| row.elapsed_ns).collect(), 1, 2);
                scoped_latency_p50_lower_in_every_pair &= p50(Lane::Paged) < p50(Lane::Control);
            }
        }
    }
    // Every raw row and every pair/scope summary precedes the decision. A
    // latency regression remains visible and is never called a speedup.
    println!(
        "{}",
        json!({"schema":"riauth.s02-measurement-decision/v2","native_point_and_value_byte_equivalence":native_equivalent,"observed_fetch_materialization_improved":fetch_materialization_improved,"scoped_latency_p50_lower_in_every_pair":scoped_latency_p50_lower_in_every_pair,"latency_grade":false,"heap_claim":false,"rss_claim":false})
    );
    assert!(
        native_equivalent,
        "security point reads or total stored bytes differ"
    );
    assert!(
        fetch_materialization_improved,
        "measured raw-fetch materialization did not improve"
    );
}

fn source_digest(group: &Value) -> String {
    let typed: Group = require(serde_json::from_value(group.clone()), "expected Group type");
    URL_SAFE_NO_PAD.encode(Sha256::digest(require(
        serde_json::to_vec(&typed),
        "expected Group serialization",
    )))
}

// Exact full-snapshot oracle: only nondeterministic audit id/time are read from
// the new event, then independently type/bound/key checked. Entire event schema,
// changes, revision and every other durable value are constructed or retained.
fn expected_audit(
    before: &Snapshot,
    after: &Snapshot,
    actor: &str,
    action: &str,
    target: &str,
    changes: Value,
    start: u64,
    end: u64,
) -> (String, Value) {
    let added: Vec<_> = after
        .iter()
        .filter(|(key, _)| key.starts_with("audit/") && !before.contains_key(*key))
        .collect();
    assert!(added.len() == 1, "writer must emit one new audit");
    let (key, row) = added[0];
    let id = row["id"].as_str().expect("audit id type");
    let uuid = require(uuid::Uuid::parse_str(id), "audit id format");
    assert!(uuid.to_string() == id, "canonical audit id");
    let at = row["at"].as_u64().expect("audit time type");
    assert!(
        start <= at && at <= end,
        "audit time outside real writer interval"
    );
    assert!(*key == format!("audit/{at:020}-{id}"), "audit key binding");
    let event = json!({"id":id,"at":at,"actor":actor,"action":action,"target":target,"run_id":null,"details":{"request_id":null,"changes":changes}});
    assert!(*row == event, "complete writer audit schema and content");
    (key.clone(), event)
}
fn admin_id(before: &Snapshot) -> &str {
    before["usernames/admin"].as_str().expect("admin stable id")
}
fn advance_revision(expected: &mut Snapshot) {
    let revision = expected["meta/revision"]
        .as_u64()
        .expect("management revision type");
    expected.insert("meta/revision".into(), json!(revision + 1));
}
fn assert_member_effect(
    before: &Snapshot,
    after: &Snapshot,
    index: usize,
    present: bool,
    start: u64,
    end: u64,
) {
    let name = group_name(index);
    let uid = user_id(0);
    let key = format!("groups/{name}");
    let old = before[&key].clone();
    let mut group: Group = require(serde_json::from_value(old.clone()), "expected old Group");
    if present {
        assert!(
            group.members.insert(uid.clone()),
            "expected membership addition"
        );
    } else {
        assert!(group.members.remove(&uid), "expected membership removal");
    }
    let new = require(serde_json::to_value(group), "expected new Group");
    let digest = source_digest(&new);
    let group_hash = crypto::digest(&name);
    let user_hash = crypto::digest(&uid);
    let mut expected = before.clone();
    expected.insert(key.clone(), new.clone());
    expected.insert(
        format!("index_group_bindings/{name}"),
        json!({"name":name,"source_digest":digest}),
    );
    expected.insert(format!("index_group_source_digests/{name}"), json!(digest));
    let user_group = format!("index_user_groups/{user_hash}/{group_hash}");
    let member = format!("index_group_members/{group_hash}/{user_hash}");
    if present {
        expected.insert(user_group, json!(name));
        expected.insert(member, json!(uid)); // Some(String) serializes as String
    } else {
        expected.remove(&user_group);
        expected.remove(&member);
    }
    advance_revision(&mut expected);
    let action = if present {
        "group.member.add"
    } else {
        "group.member.remove"
    };
    let (audit_key, event) = expected_audit(
        before,
        after,
        admin_id(before),
        action,
        &format!("{name}/{uid}"),
        json!([{"resource":key,"before":old,"after":new}]),
        start,
        end,
    );
    expected.insert(audit_key, event);
    assert!(
        expected == *after,
        "complete normal membership writer snapshot"
    );
}
fn assert_revoke_effect(
    before: &Snapshot,
    after: &Snapshot,
    credential: &str,
    start: u64,
    end: u64,
) {
    let key = "agents/s02-scoped";
    let agent: Agent = require(
        serde_json::from_value(before[key].clone()),
        "expected old Agent",
    );
    let mut old_view = agent.view();
    old_view["agent_credential"] = json!("[redacted]");
    let mut new_view = old_view.clone();
    new_view["enabled"] = json!(false);
    let mut expected = before.clone();
    expected.get_mut(key).expect("known Agent")["enabled"] = json!(false);
    assert!(
        expected
            .remove(&format!("agent_tokens/{}", crypto::digest(credential)))
            .is_some(),
        "retired exact agent token mapping"
    );
    advance_revision(&mut expected);
    let (audit_key, event) = expected_audit(
        before,
        after,
        admin_id(before),
        "agent.revoke",
        "s02-scoped",
        json!([{"resource":key,"before":old_view,"after":new_view}]),
        start,
        end,
    );
    expected.insert(audit_key, event);
    assert!(
        expected == *after,
        "complete normal revocation writer snapshot"
    );
}

// Expiry and parent liveness stay in the shared principal. The expiry row is
// a declared trusted compatibility fixture, not elapsed-TTL or remote evidence.
// Parent disable uses the real public writer; it also retires the child token.
fn authority_agent(fixture: &Fixture, id: &str, parent: Option<String>) -> Zeroizing<String> {
    token(&core_ok(
        fixture.core.create_agent(
            &fixture.credentials.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                parent,
                permissions: SELECTED
                    .iter()
                    .map(|index| Permission {
                        action: "group.read".into(),
                        resource: format!("group/{}", group_name(*index)),
                    })
                    .collect(),
            },
        ),
        "authority fixture public credential creation",
    ))
}
fn expected_user_audit(user: &User) -> Value {
    assert!(
        user.totp_secret.is_none() && user.totp_pending.is_none() && user.recovery_codes.is_empty(),
        "factor-free parent fixture required"
    );
    let mut view = require(
        serde_json::to_value(UserView::from(user)),
        "expected User view",
    );
    // public_record + fixed sensitive-name redaction at the pinned Store.
    view["password_available"] = json!("[redacted]");
    if !user.password_hash.is_empty() {
        view["password"] = json!("[redacted]");
    }
    if !user.pairwise_seed.is_empty() {
        view["pairwise_seed"] = json!("[redacted]");
    }
    view
}
fn assert_parent_disable_effect(
    before: &Snapshot,
    after: &Snapshot,
    credential: &str,
    start: u64,
    end: u64,
) {
    let user_key = format!("users/{}", user_id(0));
    let old_user: User = require(
        serde_json::from_value(before[&user_key].clone()),
        "expected enabled parent",
    );
    assert!(
        old_user.enabled && !old_user.admin,
        "enabled non-admin parent"
    );
    let mut new_user = old_user.clone();
    new_user.enabled = false;
    new_user.epoch = old_user.epoch.checked_add(1).expect("parent epoch bound");
    let agent_key = "agents/s02-parent-owned";
    let old_agent: Agent = require(
        serde_json::from_value(before[agent_key].clone()),
        "expected parent-owned Agent",
    );
    assert!(
        old_agent.enabled && old_agent.parent_user.as_deref() == Some(old_user.id.as_str()),
        "exact active parent binding"
    );
    let mut old_view = old_agent.view();
    old_view["agent_credential"] = json!("[redacted]");
    let mut new_view = old_view.clone();
    new_view["enabled"] = json!(false);
    let mut expected = before.clone();
    expected.insert(
        user_key.clone(),
        require(serde_json::to_value(&new_user), "expected disabled parent"),
    );
    expected.get_mut(agent_key).expect("known child Agent")["enabled"] = json!(false);
    assert!(
        expected
            .remove(&format!("agent_tokens/{}", crypto::digest(credential)))
            .is_some(),
        "parent disable retired exact child token"
    );
    for key in [
        "provisioning_user_generation/all",
        "user_listing_generation/all",
    ] {
        let previous = expected.get(key).and_then(Value::as_u64).unwrap_or(0);
        expected.insert(
            key.into(),
            json!(previous.checked_add(1).expect("parent generation bound")),
        );
    }
    advance_revision(&mut expected);
    // BTreeMap-backed change records sort agents before users. This fixture has
    // no owner RP/session, Windows device, grants, downstream link/job or SSF
    // stream; any unexpected durable consequence fails the COMPLETE map.
    let changes = json!([
        {"resource":agent_key,"before":old_view,"after":new_view},
        {"resource":user_key,"before":expected_user_audit(&old_user),"after":expected_user_audit(&new_user)}
    ]);
    let (audit_key, event) = expected_audit(
        before,
        after,
        admin_id(before),
        "user.update",
        &old_user.id,
        changes,
        start,
        end,
    );
    expected.insert(audit_key, event);
    assert!(
        expected == *after,
        "complete public parent-disable snapshot"
    );
}
fn authority_boundaries(fixture: &Fixture, budget: Budget) {
    budget.check();
    let expired = authority_agent(fixture, "s02-expiry-boundary", None);
    for lane in [Lane::Control, Lane::Paged] {
        checked_read(
            &fixture.core,
            lane,
            &expired,
            &expected(GROUPS, true, None),
            GROUPS,
            budget,
        );
    }
    let before = snapshot(&fixture.core);
    let key = "agents/s02-expiry-boundary";
    let mut record: Agent = require(
        serde_json::from_value(before[key].clone()),
        "expiry fixture canonical Agent",
    );
    assert!(
        record.enabled && record.parent_user.is_none(),
        "isolated expiry fixture"
    );
    let boundary = crypto::now();
    record.expires_at = boundary;
    core_ok(
        fixture
            .core
            .store
            .write(|tx| tx.put("agents", &record.id, &record)),
        "trusted canonical expiry fixture",
    );
    let mut expected_state = before.clone();
    expected_state.get_mut(key).expect("known expiry Agent")["expires_at"] = json!(boundary);
    same_snapshot(&fixture.core, &expected_state);
    assert!(
        crypto::now() >= boundary,
        "real clock moved before expiry boundary"
    );
    let same_second_observed = crypto::now() == boundary;
    for lane in [Lane::Control, Lane::Paged] {
        refused_read(&fixture.core, lane, &expired, budget);
    }
    // Token mapping remains valid and enabled; refusal therefore exercises the
    // stored expiration, rather than a missing token or an Agent disable.
    assert!(
        expected_state
            .get(&format!("agent_tokens/{}", crypto::digest(&expired)))
            .and_then(Value::as_str)
            == Some("s02-expiry-boundary"),
        "expired credential mapping retained"
    );
    let owned = authority_agent(fixture, "s02-parent-owned", Some(user_name(0)));
    for lane in [Lane::Control, Lane::Paged] {
        checked_read(
            &fixture.core,
            lane,
            &owned,
            &expected(GROUPS, true, None),
            GROUPS,
            budget,
        );
    }
    let before = snapshot(&fixture.core);
    let start = crypto::now();
    core_ok(
        fixture.core.update_user(
            &fixture.credentials.admin,
            &user_name(0),
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        ),
        "public parent disable",
    );
    let end = crypto::now();
    let after = snapshot(&fixture.core);
    assert_parent_disable_effect(&before, &after, &owned, start, end);
    for lane in [Lane::Control, Lane::Paged] {
        refused_read(&fixture.core, lane, &owned, budget);
        // Unrelated unparented authority and canonical Group bodies survive.
        checked_read(
            &fixture.core,
            lane,
            &fixture.credentials.scoped,
            &expected(GROUPS, true, None),
            GROUPS,
            budget,
        );
    }
    println!(
        "{}",
        json!({"schema":"riauth.s02-authority-boundaries/v1","expiry_source":"trusted_canonical_stored_timestamp_at_real_now","expiry_refusal_exact":true,"expiry_same_second_observed":same_second_observed,"natural_ttl_elapsed_claim":false,"public_parent_disable_exact_snapshot":true,"child_token_retired":true,"legacy_enabled_child_with_disabled_parent_runtime_claim":false,"unrelated_authority_preserved":true})
    );
    budget.check();
}

// Ordered underlying snapshot proof, deliberately distinct from the unhooked
// actual-Core overlap case. Every page uses the same Tx and captured Principal.
fn finish_snapshot(
    tx: &Tx<'_>,
    actor: &riauth::agent::Principal,
    first: Vec<(String, Group)>,
    budget: Budget,
) -> Result<Value> {
    let mut page = first;
    let mut values = Vec::new();
    loop {
        budget.check();
        let full = page.len() == PAGE;
        let after = page.last().map(|(key, _)| key.clone());
        for (_, group) in page {
            if actor.allows("group.read", &format!("group/{}", group.name)) {
                values.push(json!(group));
            }
        }
        if !full {
            break;
        }
        page = tx.scan("groups", after.as_deref(), PAGE)?;
        if page.is_empty() {
            break;
        }
    }
    Ok(Value::Array(values))
}

fn ordered_interleaving(fixture: &Fixture, revoke: bool, budget: Budget) {
    budget.check();
    let before = snapshot(&fixture.core);
    let writer_core = fixture.core.clone();
    let admin = fixture.credentials.admin.clone();
    let (start, go) = mpsc::sync_channel(0);
    let (finished, done) = mpsc::sync_channel(1);
    let worker = OwnedWorker::new(move || {
        require(
            go.recv_timeout(budget.thread_wait()),
            "ordered writer start",
        );
        let at = crypto::now();
        let result = if revoke {
            writer_core.revoke_agent(&admin, "s02-scoped")
        } else {
            writer_core.group_member(&admin, &group_name(512), &user_name(0), false)
        };
        let end = crypto::now();
        drop(writer_core);
        let _ = finished.send(result.is_ok());
        (result, at, end)
    });
    core_ok(
        fixture.core.store.read(|tx| {
            let actor = fixture.core.principal(tx, &fixture.credentials.scoped)?;
            let first = tx.scan::<Group>("groups", None, PAGE)?;
            assert!(first.len() == PAGE, "ordered first page");
            require(start.send(()), "ordered writer release");
            assert!(
                require(
                    done.recv_timeout(budget.thread_wait()),
                    "ordered writer result receipt"
                ),
                "ordered public writer did not commit"
            );
            let observed = finish_snapshot(tx, &actor, first, budget)?;
            assert!(
                observed == expected(GROUPS, true, None),
                "ordered snapshot retained whole old result"
            );
            Ok(())
        }),
        "ordered snapshot read",
    );
    let (result, at, end) = worker.join();
    core_ok(result, "ordered normal writer");
    let after = snapshot(&fixture.core);
    if revoke {
        assert_revoke_effect(&before, &after, &fixture.credentials.scoped, at, end);
        let duplicate = match fixture
            .core
            .revoke_agent(&fixture.credentials.admin, "s02-scoped")
        {
            Err(error) => error,
            Ok(_) => panic!("retired Agent accepted another revocation"),
        };
        assert!(
            duplicate.status.as_u16() == 409 && duplicate.code == "conflict",
            "repeat revocation conflict"
        );
        same_snapshot(&fixture.core, &after);
        for lane in [Lane::Control, Lane::Paged] {
            refused_read(&fixture.core, lane, &fixture.credentials.scoped, budget);
        }
    } else {
        assert_member_effect(&before, &after, 512, false, at, end);
        core_ok(
            fixture.core.group_member(
                &fixture.credentials.admin,
                &group_name(512),
                &user_name(0),
                false,
            ),
            "unchanged normal membership retry",
        );
        same_snapshot(&fixture.core, &after);
        for lane in [Lane::Control, Lane::Paged] {
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.scoped,
                &expected(GROUPS, true, Some(512)),
                GROUPS,
                budget,
            );
        }
    }
    println!(
        "{}",
        json!({"schema":"riauth.s02-ordered-snapshot/v1","operation":if revoke {"revoke"} else {"membership"},"writer_joined":true,"old_snapshot_exact":true,"fresh_request_checked":true})
    );
}

fn core_overlap(fixture: &Fixture, budget: Budget) {
    budget.check();
    let before = snapshot(&fixture.core);
    let old = expected(GROUPS, true, None);
    let new = expected(GROUPS, true, Some(512));
    let (start, go) = mpsc::sync_channel(0);
    let (finished, done): (_, Receiver<()>) = mpsc::sync_channel(1);
    let writer_core = fixture.core.clone();
    let admin = fixture.credentials.admin.clone();
    let worker = OwnedWorker::new(move || {
        require(
            go.recv_timeout(budget.thread_wait()),
            "overlap writer start",
        );
        let at = crypto::now();
        let call_start = Instant::now();
        let result = writer_core.group_member(&admin, &group_name(512), &user_name(0), false);
        let call_end = Instant::now();
        let end = crypto::now();
        drop(writer_core);
        let _ = finished.send(());
        (result, at, end, call_start, call_end)
    });
    require(start.send(()), "overlap writer release");
    let mut intervals = Vec::with_capacity(8);
    for _ in 0..8 {
        budget.check();
        let at_start = Instant::now();
        let value = core_ok(
            fixture.core.list_groups(&fixture.credentials.scoped),
            "concurrent actual Core reader",
        );
        let at_end = Instant::now();
        intervals.push((at_start, at_end));
        assert!(
            value == old || value == new,
            "concurrent Core returned mixed result"
        );
    }
    require(
        done.recv_timeout(budget.thread_wait()),
        "overlap writer completion",
    );
    let (result, at, end, call_start, call_end) = worker.join();
    core_ok(result, "concurrent normal writer");
    let after = snapshot(&fixture.core);
    assert_member_effect(&before, &after, 512, false, at, end);
    checked_read(
        &fixture.core,
        Lane::Paged,
        &fixture.credentials.scoped,
        &new,
        GROUPS,
        budget,
    );
    let observed_overlap = intervals
        .iter()
        .any(|(start, end)| *start < call_end && call_start < *end);
    println!(
        "{}",
        json!({"schema":"riauth.s02-core-overlap/v1","attempts":8,"observed_call_interval_overlap":observed_overlap,"writer_joined":true,"complete_pre_or_post_oracles":true,"between_page_injection":false})
    );
}

#[test]
#[ignore = "bounded local native materialization/security comparison; requires an explicit runtime reservation"]
fn group_listing_paging_preserves_snapshot_authority_and_measures_materialization() {
    assert!(
        riauth::context::current().is_none(),
        "context-free Core fixture required"
    );
    let budget = Budget(Instant::now() + FIXTURE_LIMIT);
    let watchdog = Watchdog::new(budget);
    telemetry_self_check();
    let root = fixture_root();
    let empty_seed = Seed::new(&root, budget);
    for count in [0, 1, 128, 129] {
        let fixture = empty_seed.open(&root, budget);
        populate(&fixture.core, count, budget);
        for lane in [Lane::Control, Lane::Paged] {
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.admin,
                &expected(count, false, None),
                count,
                budget,
            );
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.scoped,
                &expected(count, true, None),
                count,
                budget,
            );
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.empty,
                &json!([]),
                count,
                budget,
            );
            refused_read(&fixture.core, lane, "ri_agent_invalid_s02_fixture", budget);
        }
        fixture.close();
    }
    // Persisted key deliberately differs from visible name and sort order.
    // This is a trusted synthetic compatibility row, not a normal Group writer
    // acceptance claim. Existing list_groups authorizes the stored name.
    {
        let fixture = empty_seed.open(&root, budget);
        core_ok(
            fixture.core.store.write(|tx| {
                for index in 0..129 {
                    tx.put(
                        "groups",
                        &format!("s02-storage-{index:03}"),
                        &Group {
                            name: group_name(128 - index),
                            members: BTreeSet::from([user_id(0)]),
                        },
                    )?;
                }
                Ok(())
            }),
            "trusted key-name compatibility rows",
        );
        for lane in [Lane::Control, Lane::Paged] {
            let oracle = json!([{"name":group_name(0),"members":[user_id(0)]}]);
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.scoped,
                &oracle,
                129,
                budget,
            );
            let all = Value::Array(
                (0..129)
                    .rev()
                    .map(|index| json!({"name":group_name(index),"members":[user_id(0)]}))
                    .collect(),
            );
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.admin,
                &all,
                129,
                budget,
            );
        }
        fixture.close();
    }
    let dataset = empty_seed.open(&root, budget);
    populate(&dataset.core, GROUPS, budget);
    let seed = Seed::close(dataset);
    let mut observations = Vec::with_capacity(3 * 2 * 2 * SAMPLES);
    for (pair, order) in [
        [Lane::Control, Lane::Paged],
        [Lane::Paged, Lane::Control],
        [Lane::Control, Lane::Paged],
    ]
    .into_iter()
    .enumerate()
    {
        // Fresh identical CLOSED seed copies for each pair; live Core/Store
        // handles are never copied and no concurrent job runs during a sample.
        let control = seed.open(&root, budget);
        let paged = seed.open(&root, budget);
        for lane in order {
            let fixture = if lane == Lane::Control {
                &control
            } else {
                &paged
            };
            for scoped in [true, false] {
                measure(fixture, lane, pair, scoped, &mut observations, budget);
            }
        }
        // A valid credential with no matching scope retains the empty-array
        // contract even when all 513 source rows exist.
        for (fixture, lane) in [(&control, Lane::Control), (&paged, Lane::Paged)] {
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.empty,
                &json!([]),
                GROUPS,
                budget,
            );
        }
        control.close();
        paged.close();
    }
    // Write/concurrency work is strictly outside all measured call windows.
    let membership = seed.open(&root, budget);
    denied_member_write(&membership, budget);
    malformed_write_refusal(&membership, budget);
    ordered_interleaving(&membership, false, budget);
    let revoked = seed.open(&root, budget);
    ordered_interleaving(&revoked, true, budget);
    let overlap = seed.open(&root, budget);
    core_overlap(&overlap, budget);
    let authority = seed.open(&root, budget);
    authority_boundaries(&authority, budget);
    summaries(&observations); // raw numeric samples precede the cost gate
    budget.check();
    authority.close();
    overlap.close();
    revoked.close();
    membership.close();
    seed.cleanup();
    empty_seed.cleanup(); // keep shared private key alive until all copied stores close
    drop(observations);
    budget.check();
    drop(watchdog); // joined before the pre-final numeric receipt
    budget.check();
    println!(
        "{}",
        json!({"schema":"riauth.s02-group-list-result/v1","pairs":3,"order":"ABBAAB","warmups_per_lane_scope":5,"samples_per_lane_scope":10,"total_samples":120,"groups":513,"members_per_group":128,"scoped_groups":3,"all_owned_threads_joined":true,"private_fixture_directories_removed_and_checked":true,"cost_gate":"observed_raw_fetch_row_materialization","latency_grade":false,"heap_claim":false,"rss_claim":false,"pg_claim":false})
    );
    // This numeric receipt precedes final completion grading. A late stdout
    // return cannot make libtest pass; outer supervision must bound IO stalls.
    // No fixture output or file mutation follows this final clock check.
    budget.check();
}
