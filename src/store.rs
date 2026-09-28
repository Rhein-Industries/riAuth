pub mod maintenance;
mod ownership;
mod prepared;
#[doc(hidden)]
pub use ownership::{open_owner_in, require_local_in, shared_filesystem};
#[cfg(feature = "test-support")]
pub use prepared::with_prepared_pause;
use prepared::{Prepared, Range};

use crate::{
    crypto,
    error::{Error, Result},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use redb::{
    Database, ReadTransaction, ReadableDatabase, ReadableTable, ReadableTableMetadata,
    TableDefinition, WriteTransaction,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
use zeroize::Zeroizing;

const RECORDS: TableDefinition<&str, &[u8]> = TableDefinition::new("records_v1");
const FORMAT: TableDefinition<&str, &str> = TableDefinition::new("storage_format");
/// Rate-limit windows kept per node (in memory) and in the shared PostgreSQL table.
pub(crate) const RATE_WINDOWS: usize = 100_000;

/// Record effects supplied by server assembly for every writable transaction.
/// Storage owns the write order; the hook owns account security semantics.
pub(crate) trait RecordTransitions: Send + Sync {
    fn prepare_record(&self, bucket: &str, before: Option<&Value>, after: &mut Value);
    fn public_agent(&self, value: &Value) -> Result<Value>;
    fn record_transition(
        &self,
        tx: &Tx<'_>,
        bucket: &str,
        key: &str,
        before: Option<&Value>,
        after: Option<&Value>,
    ) -> Result<()>;
}

#[derive(Clone)]
pub struct Store {
    db: Backend,
    key: Option<Arc<Zeroizing<[u8; 32]>>>,
    telemetry: Arc<crate::telemetry::Telemetry>,
    transitions: Arc<dyn RecordTransitions>,
}
#[derive(Clone)]
enum Backend {
    Redb(Arc<Database>),
    Postgres(Arc<crate::postgres_store::Pool>),
}
enum Transaction<'a> {
    Prepared(&'a Store),
    Read(&'a ReadTransaction),
    Write(&'a WriteTransaction),
    Postgres(RefCell<postgres::Transaction<'a>>, bool),
}
pub struct Tx<'a> {
    transaction: Transaction<'a>,
    key: Option<&'a [u8; 32]>,
    changes: RefCell<BTreeMap<String, Value>>,
    security_events: RefCell<BTreeSet<(String, String, String)>>,
    telemetry: &'a crate::telemetry::Telemetry,
    prepared: RefCell<Option<Prepared>>,
    /// Absent only on read-only inspection handles.
    transitions: Option<&'a dyn RecordTransitions>,
}

macro_rules! read_table {
    ($self:expr, $table:ident, $body:block) => {{
        match &$self.transaction {
            Transaction::Read(tx) => {
                let $table = tx.open_table(RECORDS).map_err(Error::internal)?;
                $body
            }
            Transaction::Write(tx) => {
                let $table = tx.open_table(RECORDS).map_err(Error::internal)?;
                $body
            }
            Transaction::Postgres(_, _) | Transaction::Prepared(_) => {
                unreachable!("Remote and prepared reads are handled before redb dispatch")
            }
        }
    }};
}

impl Store {
    #[cfg(feature = "platform")]
    pub(crate) fn encrypted_at_rest(&self) -> bool {
        self.key.is_some()
    }

    pub fn telemetry(&self) -> &crate::telemetry::Telemetry {
        &self.telemetry
    }
    pub fn backend(&self) -> &'static str {
        match self.db {
            Backend::Redb(_) => "redb",
            Backend::Postgres(_) => "postgresql",
        }
    }
    pub fn ready(&self) -> Result<()> {
        if let Backend::Postgres(pool) = &self.db {
            let mut connection = pool.get()?;
            let writable: bool = connection
                .query_one("SELECT NOT pg_is_in_recovery() AND current_setting('default_transaction_read_only') = 'off'", &[])
                .map_err(crate::postgres_store::unavailable)?
                .get(0);
            if !writable {
                return Err(Error::internal("Storage is not writable"));
            }
        }
        crate::upgrade::require_active(self)?;
        crate::recovery::require_serving(self)
    }
    pub fn shared_rate_limit(
        &self,
        ip: std::net::IpAddr,
        category: &str,
        limit: u32,
    ) -> Result<bool> {
        self.shared_rate_limit_within(ip, category, limit, RATE_WINDOWS as u64)
    }
    /// A full table drops its oldest window rather than refusing addresses it has not
    /// seen, so filling it resets other clients' counts early but never locks them out.
    #[doc(hidden)]
    pub fn shared_rate_limit_within(
        &self,
        ip: std::net::IpAddr,
        category: &str,
        limit: u32,
        capacity: u64,
    ) -> Result<bool> {
        let key = crypto::digest(&format!("{ip}\0{category}"));
        self.write(|tx| {
            let at = crypto::now();
            let previous = tx.get::<(u64, u32)>("http_rates", &key)?;
            let (start, count) = previous
                .filter(|(start, _)| start.saturating_add(60) > at)
                .unwrap_or((at, 0));
            if previous.is_none() && tx.collection_count("http_rates")? >= capacity {
                tx.reclaim_expired_limits("http_rates", at)?;
                if tx.collection_count("http_rates")? >= capacity {
                    tx.evict_oldest_limit("http_rates")?;
                }
            }
            tx.put("http_rates", &key, &(start, count.saturating_add(1)))?;
            Ok(count >= limit)
        })
    }
    pub(crate) fn open_postgres_raw(
        config: crate::postgres_store::PostgresConfig,
        key: Option<Zeroizing<[u8; 32]>>,
        transitions: Arc<dyn RecordTransitions>,
    ) -> Result<Self> {
        use crate::postgres_store::{WRITE_LOCK, unavailable};
        let telemetry = Arc::new(crate::telemetry::Telemetry::default());
        let pool = crate::postgres_store::Pool::with_telemetry(config, telemetry.clone());
        let mut connection = pool.get()?;
        let mut transaction = connection.transaction().map_err(unavailable)?;
        transaction
            .query_one("SELECT pg_advisory_xact_lock($1)", &[&WRITE_LOCK])
            .map_err(unavailable)?;
        transaction.batch_execute("CREATE SCHEMA IF NOT EXISTS riauth_store; CREATE TABLE IF NOT EXISTS riauth_store.records_v1 (key bytea PRIMARY KEY, value bytea NOT NULL); CREATE TABLE IF NOT EXISTS riauth_store.storage_format (singleton boolean PRIMARY KEY DEFAULT true CHECK(singleton), encoding text NOT NULL);").map_err(unavailable)?;
        let expected = if key.is_some() {
            "aes256gcm-v1"
        } else {
            "plain-v1"
        };
        if let Some(row) = transaction
            .query_opt(
                "SELECT encoding FROM riauth_store.storage_format WHERE singleton",
                &[],
            )
            .map_err(unavailable)?
        {
            if row.get::<_, String>(0) != expected {
                return Err(Error::bad(
                    "Database encryption configuration does not match its storage format",
                ));
            }
        } else {
            let count: i64 = transaction
                .query_one("SELECT count(*) FROM riauth_store.records_v1", &[])
                .map_err(unavailable)?
                .get(0);
            if count != 0 {
                return Err(Error::bad("Missing database storage format"));
            }
            transaction
                .execute(
                    "INSERT INTO riauth_store.storage_format (encoding) VALUES ($1)",
                    &[&expected],
                )
                .map_err(unavailable)?;
        }
        transaction.commit().map_err(unavailable)?;
        drop(connection);
        Ok(Self {
            db: Backend::Postgres(pool),
            key: key.map(Arc::new),
            telemetry,
            transitions,
        })
    }
    pub(crate) fn open_with_key_raw(
        path: &Path,
        key: Option<Zeroizing<[u8; 32]>>,
        transitions: Arc<dyn RecordTransitions>,
    ) -> Result<Self> {
        let db = ownership::open_owner(path)?;
        let tx = db.begin_write().map_err(Error::internal)?;
        {
            let records = tx.open_table(RECORDS).map_err(Error::internal)?;
            let mut format = tx.open_table(FORMAT).map_err(Error::internal)?;
            let expected = if key.is_some() {
                "aes256gcm-v1"
            } else {
                "plain-v1"
            };
            let existing = format
                .get("encoding")
                .map_err(Error::internal)?
                .map(|v| v.value().to_owned());
            if let Some(existing) = existing {
                if existing != expected {
                    return Err(Error::bad(
                        "Database encryption configuration does not match its storage format",
                    ));
                }
            } else {
                if key.is_some() && !records.is_empty().map_err(Error::internal)? {
                    return Err(Error::bad(
                        "Use encrypted backup/restore to convert a plaintext database",
                    ));
                }
                format
                    .insert("encoding", expected)
                    .map_err(Error::internal)?;
            }
        }
        tx.commit().map_err(Error::internal)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(Error::internal)?;
        }
        Ok(Self {
            db: Backend::Redb(Arc::new(db)),
            key: key.map(Arc::new),
            telemetry: Arc::default(),
            transitions,
        })
    }
    /// Read the configured store without creating, formatting or writing it.
    /// `f` receives `None` when no riAuth store exists at that location. The
    /// encryption format is checked as on open.
    pub fn inspect<T>(
        config: &crate::config::Config,
        f: impl FnOnce(&'static str, Option<&Tx<'_>>) -> Result<T>,
    ) -> Result<T> {
        let key = config
            .database_key_file
            .as_deref()
            .map(crypto::read_key)
            .transpose()?;
        let expected = if key.is_some() {
            "aes256gcm-v1"
        } else {
            "plain-v1"
        };
        let mismatch =
            || Error::bad("Database encryption configuration does not match its storage format");
        let telemetry = Arc::new(crate::telemetry::Telemetry::default());
        let tx = |transaction| Tx {
            transaction,
            key: key.as_deref(),
            changes: RefCell::default(),
            security_events: RefCell::default(),
            telemetry: &telemetry,
            prepared: RefCell::default(),
            transitions: None,
        };
        if let Some(pg) = &config.postgres {
            use crate::postgres_store::unavailable;
            let pool = crate::postgres_store::Pool::with_telemetry(pg.clone(), telemetry.clone());
            let mut connection = pool.get()?;
            let mut transaction = connection
                .build_transaction()
                .isolation_level(postgres::IsolationLevel::RepeatableRead)
                .read_only(true)
                .start()
                .map_err(unavailable)?;
            let row = transaction
                .query_one(
                    "SELECT to_regclass('riauth_store.records_v1') IS NOT NULL, to_regclass('riauth_store.storage_format') IS NOT NULL",
                    &[],
                )
                .map_err(unavailable)?;
            if !row.get::<_, bool>(0) {
                return f("postgresql", None);
            }
            let encoding = if row.get::<_, bool>(1) {
                transaction
                    .query_opt(
                        "SELECT encoding FROM riauth_store.storage_format WHERE singleton",
                        &[],
                    )
                    .map_err(unavailable)?
                    .map(|row| row.get::<_, String>(0))
            } else {
                None
            };
            match encoding {
                Some(encoding) if encoding != expected => return Err(mismatch()),
                Some(_) => {}
                None => {
                    let empty: bool = transaction
                        .query_one(
                            "SELECT NOT EXISTS (SELECT 1 FROM riauth_store.records_v1)",
                            &[],
                        )
                        .map_err(unavailable)?
                        .get(0);
                    if !empty {
                        return Err(Error::bad("Missing database storage format"));
                    }
                    return f("postgresql", None);
                }
            }
            return f(
                "postgresql",
                Some(&tx(Transaction::Postgres(RefCell::new(transaction), false))),
            );
        }
        let path = config.data_dir.join("riauth.redb");
        if !path.try_exists().map_err(Error::internal)? {
            return f("redb", None);
        }
        let checked = ownership::require_local(&path)?;
        ownership::require_lock_support(&path, &checked)?;
        // Read-only: never repairs, formats or re-permissions the file.
        let db = redb::ReadOnlyDatabase::open(&path).map_err(|error| match error {
            redb::DatabaseError::RepairAborted => Error::conflict(
                "The redb store was not closed cleanly; open it with riauth to repair it first",
            ),
            error => ownership::open_error(&path, error),
        })?;
        ownership::reopened(&path, &checked, None)?;
        let transaction = db.begin_read().map_err(Error::internal)?;
        let encoding = match transaction.open_table(FORMAT) {
            Ok(table) => table
                .get("encoding")
                .map_err(Error::internal)?
                .map(|value| value.value().to_owned()),
            Err(redb::TableError::TableDoesNotExist(_)) => None,
            Err(error) => return Err(Error::internal(error)),
        };
        let empty = match transaction.open_table(RECORDS) {
            Ok(table) => table.is_empty().map_err(Error::internal)?,
            Err(redb::TableError::TableDoesNotExist(_)) => return f("redb", None),
            Err(error) => return Err(Error::internal(error)),
        };
        match encoding {
            Some(encoding) if encoding != expected => return Err(mismatch()),
            // Open would stamp a plaintext format on existing records.
            None if key.is_some() && !empty => {
                return Err(Error::bad(
                    "Use encrypted backup/restore to convert a plaintext database",
                ));
            }
            _ => {}
        }
        f("redb", Some(&tx(Transaction::Read(&transaction))))
    }
    fn key(&self) -> Option<&[u8; 32]> {
        self.key.as_ref().map(|k| &***k)
    }
    pub fn read<T>(&self, f: impl FnOnce(&Tx<'_>) -> Result<T>) -> Result<T> {
        if let Backend::Postgres(pool) = &self.db {
            let mut connection = pool.get()?;
            let transaction = connection
                .build_transaction()
                .isolation_level(postgres::IsolationLevel::RepeatableRead)
                .read_only(true)
                .start()
                .map_err(crate::postgres_store::unavailable)?;
            return f(&Tx {
                transaction: Transaction::Postgres(RefCell::new(transaction), false),
                key: self.key(),
                changes: RefCell::default(),
                security_events: RefCell::default(),
                telemetry: &self.telemetry,
                prepared: RefCell::default(),
                transitions: Some(self.transitions.as_ref()),
            });
        }
        let Backend::Redb(db) = &self.db else {
            unreachable!()
        };
        let transaction = db.begin_read().map_err(Error::internal)?;
        f(&Tx {
            transaction: Transaction::Read(&transaction),
            key: self.key(),
            changes: RefCell::default(),
            security_events: RefCell::default(),
            telemetry: &self.telemetry,
            prepared: RefCell::default(),
            transitions: Some(self.transitions.as_ref()),
        })
    }
    pub fn get<T: DeserializeOwned>(&self, bucket: &str, key: &str) -> Result<Option<T>> {
        self.read(|tx| tx.get(bucket, key))
    }
    pub fn list<T: DeserializeOwned>(&self, bucket: &str) -> Result<Vec<(String, T)>> {
        self.read(|tx| tx.list(bucket))
    }
    pub fn write<T>(&self, f: impl FnOnce(&Tx<'_>) -> Result<T>) -> Result<T> {
        if let Backend::Postgres(pool) = &self.db {
            return self.postgres_write(pool, false, f);
        }
        let Backend::Redb(db) = &self.db else {
            unreachable!()
        };
        let waiting = self.telemetry.writer_wait();
        let transaction = db.begin_write().map_err(Error::internal)?;
        drop(waiting);
        let _holding = self.telemetry.writer_hold();
        let output = f(&Tx {
            transaction: Transaction::Write(&transaction),
            key: self.key(),
            changes: RefCell::default(),
            security_events: RefCell::default(),
            telemetry: &self.telemetry,
            prepared: RefCell::default(),
            transitions: Some(self.transitions.as_ref()),
        })?;
        let committing = self.telemetry.commit.timer();
        transaction.commit().map_err(Error::internal)?;
        drop(committing);
        Ok(output)
    }
    pub fn preview<T>(&self, f: impl FnOnce(&Tx<'_>) -> Result<T>) -> Result<T> {
        if let Backend::Postgres(pool) = &self.db {
            return self.postgres_write(pool, true, f);
        }
        let Backend::Redb(db) = &self.db else {
            unreachable!()
        };
        let waiting = self.telemetry.writer_wait();
        let transaction = db.begin_write().map_err(Error::internal)?;
        drop(waiting);
        let _holding = self.telemetry.writer_hold();
        let output = f(&Tx {
            transaction: Transaction::Write(&transaction),
            key: self.key(),
            changes: RefCell::default(),
            security_events: RefCell::default(),
            telemetry: &self.telemetry,
            prepared: RefCell::default(),
            transitions: Some(self.transitions.as_ref()),
        })?;
        transaction.abort().map_err(Error::internal)?;
        Ok(output)
    }
    fn postgres_write<T>(
        &self,
        pool: &Arc<crate::postgres_store::Pool>,
        preview: bool,
        f: impl FnOnce(&Tx<'_>) -> Result<T>,
    ) -> Result<T> {
        use crate::postgres_store::{WRITE_LOCK, unavailable};
        let mut connection = pool.get()?;
        // Read committed is intentional: the snapshot must begin AFTER the writer lock.
        // Every riAuth writer holds this transaction lock, including across processes.
        let mut transaction = connection
            .build_transaction()
            .isolation_level(postgres::IsolationLevel::ReadCommitted)
            .start()
            .map_err(unavailable)?;
        let waiting = self.telemetry.writer_wait();
        transaction
            .query_one("SELECT pg_advisory_xact_lock($1)", &[&WRITE_LOCK])
            .map_err(unavailable)?;
        drop(waiting);
        let _holding = self.telemetry.writer_hold();
        let tx = Tx {
            transaction: Transaction::Postgres(RefCell::new(transaction), true),
            key: self.key(),
            changes: RefCell::default(),
            security_events: RefCell::default(),
            telemetry: &self.telemetry,
            prepared: RefCell::default(),
            transitions: Some(self.transitions.as_ref()),
        };
        let output = f(&tx)?;
        let Transaction::Postgres(transaction, _) = tx.transaction else {
            unreachable!()
        };
        if preview {
            transaction.into_inner().rollback().map_err(unavailable)?;
        } else {
            let _committing = self.telemetry.commit.timer();
            transaction.into_inner().commit().map_err(unavailable)?;
        }
        Ok(output)
    }
}

fn record_key(bucket: &str, key: &str) -> String {
    format!("{bucket}/{key}")
}
fn decode<T: DeserializeOwned>(key: Option<&[u8; 32]>, name: &str, value: &[u8]) -> Result<T> {
    match key {
        Some(key) => serde_json::from_slice(&crypto::unseal(key, name.as_bytes(), value)?)
            .map_err(Error::internal),
        None => serde_json::from_slice(value).map_err(Error::internal),
    }
}

fn oversized_snapshot_record(max_raw_bytes: usize) -> Error {
    let size = if max_raw_bytes.is_multiple_of(1024 * 1024) {
        format!("{} MiB", max_raw_bytes / (1024 * 1024))
    } else if max_raw_bytes.is_multiple_of(1024) {
        format!("{} KiB", max_raw_bytes / 1024)
    } else {
        format!("{max_raw_bytes} bytes")
    };
    Error::bad(format!(
        "Backup record exceeds snapshot page limit of {max_raw_bytes} bytes ({size})"
    ))
}

/// Replace values whose names carry secrets, passwords, tokens, hashes, or key
/// material. `[redacted]` and `[changed]` markers are preserved.
pub(crate) fn redact_audit_value(value: &mut Value) {
    redact_audit_value_at(value, 0);
}

fn redact_audit_value_at(value: &mut Value, depth: usize) {
    if depth >= 32 {
        *value = Value::String("[redacted]".into());
        return;
    }
    match value {
        Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if sensitive_audit_name(&key) {
                    let sentinel = matches!(
                        map.get(&key),
                        Some(Value::String(marker))
                            if marker == "[redacted]" || marker == "[changed]"
                    );
                    if !sentinel {
                        map.insert(key, Value::String("[redacted]".into()));
                    }
                } else if let Some(child) = map.get_mut(&key) {
                    redact_audit_value_at(child, depth + 1);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_audit_value_at(item, depth + 1);
            }
        }
        _ => {}
    }
}

pub(crate) fn sensitive_audit_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    const PARTS: &[&str] = &[
        "secret",
        "password",
        "token",
        "hash",
        "totp",
        "recovery",
        "seed",
        "private_key",
        "key_material",
    ];
    PARTS.iter().any(|part| name.contains(part))
        || matches!(
            name.as_str(),
            "authorization" | "proxy-authorization" | "cookie"
        )
        || name.contains("authorization_header")
        || name.contains("auth_header")
}

fn public_record(
    bucket: &str,
    value: Option<&Value>,
    transitions: &dyn RecordTransitions,
) -> Result<Value> {
    let Some(value) = value else {
        return Ok(Value::Null);
    };
    Ok(match bucket {
        "users" => serde_json::json!(crate::model::UserView::from(
            &serde_json::from_value::<crate::model::User>(value.clone())
                .map_err(Error::internal)?
        )),
        "clients" => serde_json::from_value::<crate::model::Client>(value.clone())
            .map_err(Error::internal)?
            .view(),
        "agents" => transitions.public_agent(value)?,
        _ => value.clone(),
    })
}

fn credential_fields(bucket: &str) -> &'static [(&'static str, &'static str)] {
    match bucket {
        "users" => &[
            ("password_hash", "password"),
            ("totp_secret", "totp"),
            ("totp_pending", "totp_pending"),
            ("recovery_codes", "recovery_codes"),
            ("pairwise_seed", "pairwise_seed"),
        ],
        "clients" => &[("secret_hash", "client_secret")],
        "agents" => &[("token_hash", "agent_credential")],
        _ => &[],
    }
}

fn credential_material(value: Option<&Value>) -> Option<&Value> {
    match value {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) if text.is_empty() => None,
        Some(Value::Array(items)) if items.is_empty() => None,
        Some(other) => Some(other),
    }
}

fn stamp_credential(view: &mut Value, label: &str, marker: &str) {
    if let Some(map) = view.as_object_mut() {
        map.insert(label.to_owned(), Value::String(marker.into()));
    }
}

/// Compare credential fields without copying them into the audit record.
fn credential_markers(
    bucket: &str,
    before_raw: Option<&Value>,
    after_raw: Option<&Value>,
    before: &mut Value,
    after: &mut Value,
) -> Vec<&'static str> {
    let mut changed = Vec::new();
    for (field, label) in credential_fields(bucket) {
        let left = credential_material(before_raw.and_then(|value| value.get(*field)));
        let right = credential_material(after_raw.and_then(|value| value.get(*field)));
        if left.is_none() && right.is_none() {
            continue;
        }
        if left == right {
            stamp_credential(before, label, "[redacted]");
            stamp_credential(after, label, "[redacted]");
            continue;
        }
        changed.push(*label);
        if left.is_some() {
            stamp_credential(before, label, "[redacted]");
        }
        if right.is_some() {
            stamp_credential(after, label, "[changed]");
        }
    }
    changed
}

fn merge_changed_credentials(change: &mut Value, labels: &[&str]) {
    let mut existing = change
        .get("changed_credentials")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for label in labels {
        let value = Value::String((*label).to_owned());
        if !existing.contains(&value) {
            existing.push(value);
        }
    }
    change["changed_credentials"] = Value::Array(existing);
}

impl Tx<'_> {
    pub fn changes(&self) -> Value {
        serde_json::json!(self.changes.borrow().values().collect::<Vec<_>>())
    }
    fn record_change(
        &self,
        bucket: &str,
        key: &str,
        after: Option<Value>,
        transitions: &dyn RecordTransitions,
    ) -> Result<()> {
        if bucket == "source_secrets" {
            return self.record_redacted_secret(key, after.is_some());
        }
        if ![
            "users",
            "clients",
            "groups",
            "agents",
            "sources",
            "windows_devices",
        ]
        .contains(&bucket)
        {
            return Ok(());
        }
        let name = record_key(bucket, key);
        let before_raw = self.get::<Value>(bucket, key)?;
        let mut before_view = public_record(bucket, before_raw.as_ref(), transitions)?;
        let mut after_view = public_record(bucket, after.as_ref(), transitions)?;
        redact_audit_value(&mut before_view);
        redact_audit_value(&mut after_view);
        let changed = credential_markers(
            bucket,
            before_raw.as_ref(),
            after.as_ref(),
            &mut before_view,
            &mut after_view,
        );
        let mut changes = self.changes.borrow_mut();
        let entry = changes
            .entry(name.clone())
            .or_insert_with(|| serde_json::json!({"resource": name, "before": before_view}));
        entry["after"] = after_view;
        if !changed.is_empty() {
            merge_changed_credentials(entry, &changed);
        }
        Ok(())
    }

    /// Existence only. The secret bytes are never decoded into the change log.
    fn record_redacted_secret(&self, key: &str, has_after: bool) -> Result<()> {
        let name = record_key("source_secrets", key);
        let existed = self.raw_get(&name)?.is_some();
        if !existed && !has_after {
            return Ok(());
        }
        let before = if existed {
            Value::String("[redacted]".into())
        } else {
            Value::Null
        };
        let after = if has_after {
            Value::String("[redacted]".into())
        } else {
            Value::Null
        };
        let mut changes = self.changes.borrow_mut();
        let entry = changes.entry(name.clone()).or_insert_with(|| {
            serde_json::json!({
                "resource": name,
                "before": before,
                "changed_credentials": ["source_secret"],
            })
        });
        entry["after"] = after;
        if entry.get("changed_credentials").is_none() {
            entry["changed_credentials"] = serde_json::json!(["source_secret"]);
        }
        Ok(())
    }

    fn writer(&self) -> Result<&WriteTransaction> {
        match &self.transaction {
            Transaction::Write(tx) => Ok(tx),
            Transaction::Read(_) | Transaction::Prepared(_) => Err(Error::internal(
                "Mutation attempted inside a read transaction",
            )),
            Transaction::Postgres(_, _) => {
                Err(Error::internal("redb writer requested for PostgreSQL"))
            }
        }
    }
    pub fn get<T: DeserializeOwned>(&self, bucket: &str, key: &str) -> Result<Option<T>> {
        let name = record_key(bucket, key);
        self.raw_get(&name)?
            .map(|bytes| decode(self.key, &name, &bytes))
            .transpose()
    }
    fn read_context(&self) -> crate::telemetry::ReadContext {
        use crate::telemetry::ReadContext;
        match self.transaction {
            Transaction::Prepared(_) => ReadContext::Prepared,
            Transaction::Write(_) | Transaction::Postgres(_, true) => ReadContext::Writer,
            Transaction::Read(_) | Transaction::Postgres(_, false) => ReadContext::Read,
        }
    }
    fn raw_get_base(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let bytes = if let Transaction::Prepared(store) = self.transaction {
            store.read(|tx| tx.raw_get_fetch(name))?
        } else {
            self.raw_get_fetch(name)?
        };
        self.telemetry
            .reads
            .point(self.read_context(), bytes.as_ref().map_or(0, Vec::len));
        Ok(bytes)
    }
    fn raw_get_fetch(&self, name: &str) -> Result<Option<Vec<u8>>> {
        if let Transaction::Postgres(transaction, _) = &self.transaction {
            return transaction
                .borrow_mut()
                .query_opt(
                    "SELECT value FROM riauth_store.records_v1 WHERE key=$1",
                    &[&name.as_bytes()],
                )
                .map_err(crate::postgres_store::unavailable)?
                .map(|row| Ok(row.get(0)))
                .transpose();
        }
        read_table!(self, table, {
            Ok(table
                .get(name)
                .map_err(Error::internal)?
                .map(|value| value.value().to_vec()))
        })
    }
    pub fn put<T: Serialize>(&self, bucket: &str, key: &str, value: &T) -> Result<()> {
        let transitions = self.transitions.ok_or_else(|| {
            Error::internal("Mutation attempted inside a read transaction")
        })?;
        let mut public = serde_json::to_value(value).map_err(Error::internal)?;
        let before = if matches!(bucket, "users" | "groups" | "passkeys") {
            self.get::<Value>(bucket, key)?
        } else {
            None
        };
        transitions.prepare_record(bucket, before.as_ref(), &mut public);
        self.update_indexes(bucket, key, Some(&public))?;
        self.record_change(bucket, key, Some(public.clone()), transitions)?;
        self.import_record(bucket, key, &public)?;
        transitions.record_transition(self, bucket, key, before.as_ref(), Some(&public))
    }
    pub(crate) fn import_record<T: Serialize>(
        &self,
        bucket: &str,
        key: &str,
        value: &T,
    ) -> Result<()> {
        let name = record_key(bucket, key);
        let plain = Zeroizing::new(serde_json::to_vec(value).map_err(Error::internal)?);
        let bytes = match self.key {
            Some(key) => crypto::seal(key, name.as_bytes(), &plain)?,
            None => plain.to_vec(),
        };
        self.raw_put(&name, Some(bytes))
    }
    pub fn delete(&self, bucket: &str, key: &str) -> Result<()> {
        let transitions = self.transitions.ok_or_else(|| {
            Error::internal("Mutation attempted inside a read transaction")
        })?;
        let before = if matches!(bucket, "users" | "groups" | "passkeys") {
            self.get::<Value>(bucket, key)?
        } else {
            None
        };
        self.update_indexes(bucket, key, None)?;
        self.record_change(bucket, key, None, transitions)?;
        self.raw_put(&record_key(bucket, key), None)?;
        transitions.record_transition(self, bucket, key, before.as_ref(), None)
    }

    pub(crate) fn mark_security_event(&self, user: &str, event: &str, credential: &str) -> bool {
        self.security_events
            .borrow_mut()
            .insert((user.into(), event.into(), credential.into()))
    }

    fn raw_put(&self, name: &str, bytes: Option<Vec<u8>>) -> Result<()> {
        if self.stage(name, &bytes)? {
            return Ok(());
        }
        if let Transaction::Postgres(transaction, writable) = &self.transaction {
            if !writable {
                return Err(Error::internal(
                    "Mutation attempted inside a read transaction",
                ));
            }
            match bytes {
                Some(bytes) => {
                    transaction.borrow_mut().execute("INSERT INTO riauth_store.records_v1 (key,value) VALUES ($1,$2) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value", &[&name.as_bytes(), &bytes]).map_err(crate::postgres_store::unavailable)?;
                }
                None => {
                    transaction
                        .borrow_mut()
                        .execute(
                            "DELETE FROM riauth_store.records_v1 WHERE key=$1",
                            &[&name.as_bytes()],
                        )
                        .map_err(crate::postgres_store::unavailable)?;
                }
            }
            return Ok(());
        }
        let mut table = self
            .writer()?
            .open_table(RECORDS)
            .map_err(Error::internal)?;
        match bytes {
            Some(bytes) => {
                table
                    .insert(name, bytes.as_slice())
                    .map_err(Error::internal)?;
            }
            None => {
                table.remove(name).map_err(Error::internal)?;
            }
        }
        Ok(())
    }
    pub fn list<T: DeserializeOwned>(&self, bucket: &str) -> Result<Vec<(String, T)>> {
        self.scan(bucket, None, usize::MAX)
    }
    pub fn scan<T: DeserializeOwned>(
        &self,
        bucket: &str,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, T)>> {
        self.scan_direction(bucket, after, limit, false)
    }
    pub fn scan_reverse<T: DeserializeOwned>(
        &self,
        bucket: &str,
        before: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, T)>> {
        self.scan_direction(bucket, before, limit, true)
    }
    fn scan_direction<T: DeserializeOwned>(
        &self,
        bucket: &str,
        cursor: Option<&str>,
        limit: usize,
        reverse: bool,
    ) -> Result<Vec<(String, T)>> {
        let prefix = format!("{bucket}/");
        let start = if reverse {
            prefix.clone()
        } else {
            cursor
                .map(|v| format!("{prefix}{v}\0"))
                .unwrap_or_else(|| prefix.clone())
        };
        let end = if reverse {
            cursor
                .map(|v| format!("{prefix}{v}"))
                .unwrap_or_else(|| format!("{bucket}0"))
        } else {
            format!("{bucket}0")
        };
        if start >= end || limit == 0 {
            return Ok(Vec::new());
        }
        self.scan_records(Range {
            start,
            end,
            limit,
            reverse,
        })?
        .into_iter()
        .map(|(name, bytes)| {
            Ok((
                name[prefix.len()..].to_owned(),
                decode(self.key, &name, &bytes)?,
            ))
        })
        .collect()
    }
    fn raw_scan_base(&self, range: &Range) -> Result<Vec<(String, Vec<u8>)>> {
        let records = if let Transaction::Prepared(store) = self.transaction {
            store.read(|tx| tx.raw_scan_fetch(range))?
        } else {
            self.raw_scan_fetch(range)?
        };
        self.telemetry
            .scanned_records
            .fetch_add(records.len() as u64, std::sync::atomic::Ordering::Relaxed);
        self.telemetry.reads.scan(
            self.read_context(),
            range.limit != usize::MAX,
            records.len(),
            records.iter().map(|(_, bytes)| bytes.len()).sum(),
        );
        Ok(records)
    }
    fn raw_scan_fetch(&self, range: &Range) -> Result<Vec<(String, Vec<u8>)>> {
        if let Transaction::Postgres(transaction, _) = &self.transaction {
            let limit = i64::try_from(range.limit).unwrap_or(i64::MAX);
            let query = if range.reverse {
                "SELECT key,value FROM riauth_store.records_v1 WHERE key >= $1 AND key < $2 ORDER BY key DESC LIMIT $3"
            } else {
                "SELECT key,value FROM riauth_store.records_v1 WHERE key >= $1 AND key < $2 ORDER BY key LIMIT $3"
            };
            return transaction
                .borrow_mut()
                .query(
                    query,
                    &[&range.start.as_bytes(), &range.end.as_bytes(), &limit],
                )
                .map_err(crate::postgres_store::unavailable)?
                .into_iter()
                .map(|row| {
                    Ok((
                        String::from_utf8(row.get(0)).map_err(Error::internal)?,
                        row.get(1),
                    ))
                })
                .collect();
        }
        read_table!(self, table, {
            let iter = table
                .range(range.start.as_str()..range.end.as_str())
                .map_err(Error::internal)?;
            let iter: Box<dyn Iterator<Item = _>> = if range.reverse {
                Box::new(iter.rev().take(range.limit))
            } else {
                Box::new(iter.take(range.limit))
            };
            iter.map(|entry| {
                let (key, value) = entry.map_err(Error::internal)?;
                Ok((key.value().to_owned(), value.value().to_vec()))
            })
            .collect()
        })
    }
    /// Read only the next record key in storage order. Recovery uses this to
    /// discover collections without materializing an unbounded record value.
    pub(crate) fn snapshot_next_key(&self, after: Option<&str>) -> Result<Option<String>> {
        if let Transaction::Postgres(transaction, _) = &self.transaction {
            let rows = if let Some(after) = after {
                transaction.borrow_mut().query(
                    "SELECT key FROM riauth_store.records_v1 WHERE key > $1 ORDER BY key LIMIT 1",
                    &[&after.as_bytes()],
                )
            } else {
                transaction.borrow_mut().query(
                    "SELECT key FROM riauth_store.records_v1 ORDER BY key LIMIT 1",
                    &[],
                )
            }
            .map_err(crate::postgres_store::unavailable)?;
            return rows
                .into_iter()
                .next()
                .map(|row| String::from_utf8(row.get(0)).map_err(Error::internal))
                .transpose();
        }
        if matches!(&self.transaction, Transaction::Prepared(_)) {
            return Err(Error::internal(
                "Recovery collection scan requires a storage transaction",
            ));
        }
        read_table!(self, table, {
            let start = after.map(|key| format!("{key}\0")).unwrap_or_default();
            let mut entries = table.range(start.as_str()..).map_err(Error::internal)?;
            entries
                .next()
                .map(|entry| {
                    let (key, _) = entry.map_err(Error::internal)?;
                    Ok(key.value().to_owned())
                })
                .transpose()
        })
    }
    /// PostgreSQL identity of this record table; `None` on redb. A physical copy
    /// or PITR of the same cluster keeps every value, so this detects only
    /// logical restores into another cluster, database or table.
    pub(crate) fn postgres_lineage(&self) -> Result<Option<crate::recovery::Lineage>> {
        use crate::postgres_store::unavailable;
        let Transaction::Postgres(transaction, _) = &self.transaction else {
            return Ok(None);
        };
        let mut transaction = transaction.borrow_mut();
        let row = transaction
            .query_one(
                "SELECT (SELECT oid FROM pg_database WHERE datname = current_database()), 'riauth_store.records_v1'::regclass::oid",
                &[],
            )
            .map_err(unavailable)?;
        let (database_oid, records_oid): (u32, u32) = (row.get(0), row.get(1));
        // Managed services may revoke or omit pg_control_system(); record it as unavailable.
        let mut probe = transaction.transaction().map_err(unavailable)?;
        let system_identifier = match probe.query_one(
            "SELECT system_identifier::text FROM pg_control_system()",
            &[],
        ) {
            Ok(row) => {
                let value: String = row.get(0);
                probe.commit().map_err(unavailable)?;
                Some(value)
            }
            // Any server-reported refusal; a lost connection still fails below.
            Err(error) if error.code().is_some() => {
                probe.rollback().map_err(unavailable)?;
                None
            }
            Err(error) => return Err(unavailable(error)),
        };
        Ok(Some(crate::recovery::Lineage {
            system_identifier,
            database_oid,
            records_oid,
        }))
    }
    /// Other riAuth sessions connected to this database; `None` on redb, whose file
    /// lock already excludes a concurrent server.
    pub(crate) fn postgres_other_clients(&self) -> Result<Option<i64>> {
        let Transaction::Postgres(transaction, _) = &self.transaction else {
            return Ok(None);
        };
        Ok(Some(
            transaction
                .borrow_mut()
                .query_one(
                    "SELECT count(*) FROM pg_stat_activity WHERE datname = current_database() AND application_name = 'riauth' AND pid <> pg_backend_pid()",
                    &[],
                )
                .map_err(crate::postgres_store::unavailable)?
                .get(0),
        ))
    }
    /// Freeze the PostgreSQL record table while an offline edition transition
    /// compares its planned snapshot and stamps both activation markers. The
    /// ordinary riAuth writer lock excludes cooperating processes; this table
    /// lock also excludes direct SQL writes until the transaction commits.
    pub(crate) fn lock_records_for_transition(&self) -> Result<()> {
        if let Transaction::Postgres(transaction, true) = &self.transaction {
            transaction
                .borrow_mut()
                .batch_execute("LOCK TABLE riauth_store.records_v1 IN SHARE ROW EXCLUSIVE MODE")
                .map_err(crate::postgres_store::unavailable)?;
        }
        Ok(())
    }

    /// Bounded-memory fingerprint of every persisted record, including opaque
    /// authority and revision data. Read inspection has one consistent snapshot;
    /// a transition writer calls this only after locking out other writers.
    pub(crate) fn snapshot_digest(&self) -> Result<String> {
        let mut hash = Sha256::new();
        hash.update(b"riauth.edition-transition-store/v1\0");
        let mut after = None;
        while let Some(name) = self.snapshot_next_key(after.as_deref())? {
            let value = self.raw_get_base(&name)?.ok_or_else(|| {
                Error::conflict("Store changed during edition transition inspection")
            })?;
            hash.update((name.len() as u64).to_be_bytes());
            hash.update(name.as_bytes());
            hash.update((value.len() as u64).to_be_bytes());
            hash.update(&value);
            after = Some(name);
        }
        Ok(URL_SAFE_NO_PAD.encode(hash.finalize()))
    }
    /// One page of the whole keyspace, strictly after `after`. The sum of
    /// stored key and value bytes never exceeds `max_raw_bytes`; an oversized
    /// first record is an error, so callers cannot mistake it for EOF.
    /// Repeat inside the same read transaction for a consistent scan.
    pub(crate) fn snapshot_page_bounded(
        &self,
        after: Option<&str>,
        limit: usize,
        max_raw_bytes: usize,
    ) -> Result<Vec<(String, Value)>> {
        let page = self.snapshot_page_raw_bounded(after, limit, max_raw_bytes)?;
        self.telemetry
            .snapshot_records
            .fetch_add(page.len() as u64, std::sync::atomic::Ordering::Relaxed);
        page.into_iter()
            .map(|(name, bytes)| {
                let value = decode(self.key, &name, &bytes)?;
                Ok((name, value))
            })
            .collect()
    }
    fn snapshot_page_raw_bounded(
        &self,
        after: Option<&str>,
        limit: usize,
        max_raw_bytes: usize,
    ) -> Result<Vec<(String, Vec<u8>)>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        if max_raw_bytes == 0 {
            return Err(Error::bad("Snapshot page byte limit must be positive"));
        }
        if self.prepared.borrow().is_some()
            || !matches!(
                self.transaction,
                Transaction::Read(_) | Transaction::Postgres(_, false)
            )
        {
            return Err(Error::internal(
                "Paged snapshots require an ordinary read transaction",
            ));
        }
        if let Transaction::Postgres(transaction, _) = &self.transaction {
            // Read metadata first. A LIMIT-only value query could materialize
            // 128 arbitrarily large legacy values before Rust can reject them.
            // Store::read holds a REPEATABLE READ transaction across both queries.
            let limit = i64::try_from(limit.min(maintenance::PAGE)).map_err(Error::internal)?;
            let max_bytes = i64::try_from(max_raw_bytes).map_err(Error::internal)?;
            let mut transaction = transaction.borrow_mut();
            let metadata = if let Some(after) = after {
                transaction.query(
                    "WITH candidates AS MATERIALIZED (SELECT key, octet_length(key)::bigint + octet_length(value)::bigint AS record_bytes FROM riauth_store.records_v1 WHERE key > $1 ORDER BY key LIMIT $2), sized AS (SELECT key, record_bytes, sum(record_bytes) OVER (ORDER BY key) AS total_bytes, row_number() OVER (ORDER BY key) AS ordinal FROM candidates) SELECT CASE WHEN total_bytes <= $3::bigint THEN key ELSE NULL END, record_bytes FROM sized WHERE total_bytes <= $3::bigint OR ordinal = 1 ORDER BY ordinal",
                    &[&after.as_bytes(), &limit, &max_bytes],
                )
            } else {
                transaction.query(
                    "WITH candidates AS MATERIALIZED (SELECT key, octet_length(key)::bigint + octet_length(value)::bigint AS record_bytes FROM riauth_store.records_v1 ORDER BY key LIMIT $1), sized AS (SELECT key, record_bytes, sum(record_bytes) OVER (ORDER BY key) AS total_bytes, row_number() OVER (ORDER BY key) AS ordinal FROM candidates) SELECT CASE WHEN total_bytes <= $2::bigint THEN key ELSE NULL END, record_bytes FROM sized WHERE total_bytes <= $2::bigint OR ordinal = 1 ORDER BY ordinal",
                    &[&limit, &max_bytes],
                )
            }
            .map_err(crate::postgres_store::unavailable)?;
            let keys: Vec<Vec<u8>> = metadata
                .into_iter()
                .map(|row| {
                    row.get::<_, Option<Vec<u8>>>(0)
                        .ok_or_else(|| oversized_snapshot_record(max_raw_bytes))
                })
                .collect::<Result<_>>()?;
            let (Some(first), Some(last)) = (keys.first(), keys.last()) else {
                return Ok(Vec::new());
            };
            let rows = transaction
                .query(
                    "SELECT key,value FROM riauth_store.records_v1 WHERE key >= $1 AND key <= $2 ORDER BY key",
                    &[first, last],
                )
                .map_err(crate::postgres_store::unavailable)?;
            if rows.len() != keys.len() {
                return Err(Error::internal("Snapshot changed during page read"));
            }
            return rows
                .into_iter()
                .zip(keys)
                .map(|(row, expected)| {
                    let key: Vec<u8> = row.get(0);
                    if key != expected {
                        return Err(Error::internal("Snapshot changed during page read"));
                    }
                    Ok((String::from_utf8(key).map_err(Error::internal)?, row.get(1)))
                })
                .collect();
        }
        read_table!(self, table, {
            // `\0` is the inclusive successor used by bucket scans; it excludes `after`.
            let start = after.map(|key| format!("{key}\0")).unwrap_or_default();
            let iter = table.range(start.as_str()..).map_err(Error::internal)?;
            let mut page = Vec::new();
            let mut used = 0usize;
            for entry in iter.take(limit.min(maintenance::PAGE)) {
                let (key, value) = entry.map_err(Error::internal)?;
                let bytes = key
                    .value()
                    .len()
                    .checked_add(value.value().len())
                    .and_then(|bytes| used.checked_add(bytes))
                    .ok_or_else(|| oversized_snapshot_record(max_raw_bytes))?;
                if bytes > max_raw_bytes {
                    if page.is_empty() {
                        return Err(oversized_snapshot_record(max_raw_bytes));
                    }
                    break;
                }
                used = bytes;
                page.push((key.value().to_owned(), value.value().to_vec()));
            }
            Ok(page)
        })
    }
    pub fn snapshot(&self) -> Result<BTreeMap<String, Value>> {
        let records = self.snapshot_all()?;
        self.telemetry
            .snapshot_records
            .fetch_add(records.len() as u64, std::sync::atomic::Ordering::Relaxed);
        Ok(records)
    }
    fn snapshot_all(&self) -> Result<BTreeMap<String, Value>> {
        if self.prepared.borrow().is_some() {
            return Err(Error::internal(
                "Full snapshots require an ordinary read transaction",
            ));
        }
        if let Transaction::Postgres(transaction, _) = &self.transaction {
            return transaction
                .borrow_mut()
                .query(
                    "SELECT key,value FROM riauth_store.records_v1 ORDER BY key",
                    &[],
                )
                .map_err(crate::postgres_store::unavailable)?
                .into_iter()
                .map(|row| {
                    let name =
                        std::str::from_utf8(row.get::<_, &[u8]>(0)).map_err(Error::internal)?;
                    Ok((
                        name.to_owned(),
                        decode(self.key, name, row.get::<_, &[u8]>(1))?,
                    ))
                })
                .collect();
        }
        read_table!(self, table, {
            table
                .iter()
                .map_err(Error::internal)?
                .map(|entry| {
                    let (key, value) = entry.map_err(Error::internal)?;
                    Ok((
                        key.value().to_owned(),
                        decode(self.key, key.value(), value.value())?,
                    ))
                })
                .collect()
        })
    }
}

#[cfg(test)]
mod snapshot_paging_tests {
    use super::*;

    #[test]
    fn page_budget_counts_keys_and_values_before_decoding() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("store.redb")).unwrap();
        store
            .write(|tx| {
                tx.put("page", "a", &"alpha")?;
                tx.put("page", "b", &"bravo")?;
                Ok(())
            })
            .unwrap();
        let first_bytes = "page/a".len() + serde_json::to_vec("alpha").unwrap().len();
        let second_bytes = "page/b".len() + serde_json::to_vec("bravo").unwrap().len();
        store
            .read(|tx| {
                let first = tx.snapshot_page_bounded(None, 128, first_bytes)?;
                assert_eq!(
                    first,
                    vec![("page/a".into(), Value::String("alpha".into()))]
                );
                let second = tx.snapshot_page_bounded(Some("page/a"), 128, second_bytes)?;
                assert_eq!(
                    second,
                    vec![("page/b".into(), Value::String("bravo".into()))]
                );
                let both = tx.snapshot_page_bounded(None, 128, first_bytes + second_bytes)?;
                assert_eq!(both.len(), 2);
                assert!(tx.snapshot_page_bounded(Some("page/b"), 128, 1)?.is_empty());
                Ok(())
            })
            .unwrap();
        let error = store
            .read(|tx| tx.snapshot_page_bounded(None, 128, first_bytes - 1))
            .unwrap_err();
        assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
        assert!(error.message.contains("snapshot page limit"));
        let error = store
            .read(|tx| tx.snapshot_page_bounded(Some("page/a"), 128, second_bytes - 1))
            .unwrap_err();
        assert!(error.message.contains("snapshot page limit"));
    }

    #[test]
    fn oversized_legacy_value_is_rejected_before_json_decode() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("store.redb")).unwrap();
        let Backend::Redb(db) = &store.db else {
            unreachable!()
        };
        let transaction = db.begin_write().unwrap();
        {
            let mut table = transaction.open_table(RECORDS).unwrap();
            table
                .insert("legacy/oversized", vec![b'!'; 4096].as_slice())
                .unwrap();
        }
        transaction.commit().unwrap();
        let error = store
            .read(|tx| tx.snapshot_page_bounded(None, 128, 1024))
            .unwrap_err();
        assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
        assert!(error.message.contains("snapshot page limit"));
    }

    #[test]
    fn pages_keep_one_read_snapshot_during_a_concurrent_commit() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("store.redb")).unwrap();
        store.write(|tx| tx.put("page", "a", &1u8)).unwrap();
        store.write(|tx| tx.put("page", "c", &3u8)).unwrap();
        store
            .read(|tx| {
                let first = tx.snapshot_page_bounded(None, 1, 1024)?;
                assert_eq!(first[0].0, "page/a");
                store.write(|writer| writer.put("page", "b", &2u8))?;
                let second = tx.snapshot_page_bounded(Some(&first[0].0), 1, 1024)?;
                assert_eq!(second[0].0, "page/c");
                Ok(())
            })
            .unwrap();
        assert_eq!(
            store
                .read(|tx| tx.snapshot_page_bounded(Some("page/a"), 1, 1024))
                .unwrap()[0]
                .0,
            "page/b"
        );
    }
}
