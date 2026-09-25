pub mod maintenance;
mod prepared;
use prepared::{Prepared, Range};

use crate::{
    crypto,
    error::{Error, Result},
};
use redb::{
    Database, ReadTransaction, ReadableDatabase, ReadableTable, ReadableTableMetadata,
    TableDefinition, WriteTransaction,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
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

#[derive(Clone)]
pub struct Store {
    db: Backend,
    key: Option<Arc<Zeroizing<[u8; 32]>>>,
    telemetry: Arc<crate::telemetry::Telemetry>,
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
    pub(crate) fn encrypted_at_rest(&self) -> bool {
        self.key.is_some()
    }

    pub fn telemetry(&self) -> &crate::telemetry::Telemetry {
        &self.telemetry
    }
    pub fn from_config(config: &crate::config::Config) -> Result<Self> {
        let key = config
            .database_key_file
            .as_deref()
            .map(crypto::read_key)
            .transpose()?;
        if let Some(pg) = &config.postgres {
            Self::open_postgres(pg.clone(), key)
        } else {
            Self::open_with_key(&config.data_dir.join("riauth.redb"), key)
        }
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
        if self.get::<u32>("meta", "schema")? != Some(crate::upgrade::SCHEMA) {
            return Err(Error::internal("Storage schema is not ready"));
        }
        Ok(())
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
    pub fn open_postgres(
        config: crate::postgres_store::PostgresConfig,
        key: Option<Zeroizing<[u8; 32]>>,
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
        })
    }
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with_key(path, None)
    }
    pub fn open_with_key(path: &Path, key: Option<Zeroizing<[u8; 32]>>) -> Result<Self> {
        let db = Database::create(path).map_err(Error::internal)?;
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
        })
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
        let waiting = self.telemetry.write_wait.timer();
        let transaction = db.begin_write().map_err(Error::internal)?;
        drop(waiting);
        let _holding = self.telemetry.write_hold.timer();
        let output = f(&Tx {
            transaction: Transaction::Write(&transaction),
            key: self.key(),
            changes: RefCell::default(),
            security_events: RefCell::default(),
            telemetry: &self.telemetry,
            prepared: RefCell::default(),
        })?;
        transaction.commit().map_err(Error::internal)?;
        Ok(output)
    }
    pub fn preview<T>(&self, f: impl FnOnce(&Tx<'_>) -> Result<T>) -> Result<T> {
        if let Backend::Postgres(pool) = &self.db {
            return self.postgres_write(pool, true, f);
        }
        let Backend::Redb(db) = &self.db else {
            unreachable!()
        };
        let waiting = self.telemetry.write_wait.timer();
        let transaction = db.begin_write().map_err(Error::internal)?;
        drop(waiting);
        let _holding = self.telemetry.write_hold.timer();
        let output = f(&Tx {
            transaction: Transaction::Write(&transaction),
            key: self.key(),
            changes: RefCell::default(),
            security_events: RefCell::default(),
            telemetry: &self.telemetry,
            prepared: RefCell::default(),
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
        let waiting = self.telemetry.write_wait.timer();
        transaction
            .query_one("SELECT pg_advisory_xact_lock($1)", &[&WRITE_LOCK])
            .map_err(unavailable)?;
        drop(waiting);
        let _holding = self.telemetry.write_hold.timer();
        let tx = Tx {
            transaction: Transaction::Postgres(RefCell::new(transaction), true),
            key: self.key(),
            changes: RefCell::default(),
            security_events: RefCell::default(),
            telemetry: &self.telemetry,
            prepared: RefCell::default(),
        };
        let output = f(&tx)?;
        let Transaction::Postgres(transaction, _) = tx.transaction else {
            unreachable!()
        };
        if preview {
            transaction.into_inner().rollback().map_err(unavailable)?;
        } else {
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

fn public_record(bucket: &str, value: Option<&Value>) -> Result<Value> {
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
        "agents" => serde_json::from_value::<crate::agent::Agent>(value.clone())
            .map_err(Error::internal)?
            .view(),
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
    fn record_change(&self, bucket: &str, key: &str, after: Option<Value>) -> Result<()> {
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
        let mut before_view = public_record(bucket, before_raw.as_ref())?;
        let mut after_view = public_record(bucket, after.as_ref())?;
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
    fn raw_get_base(&self, name: &str) -> Result<Option<Vec<u8>>> {
        if let Transaction::Prepared(store) = self.transaction {
            return store.read(|tx| tx.raw_get_base(name));
        }
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
        let mut public = serde_json::to_value(value).map_err(Error::internal)?;
        let before = if matches!(bucket, "users" | "passkeys") {
            self.get::<Value>(bucket, key)?
        } else {
            None
        };
        if bucket == "users"
            && let Some(old) = &before
            && old["enabled"] != public["enabled"]
        {
            // Account state changes revoke sessions even if a caller omitted its
            // epoch bump. Re-enabling a legacy snapshot cannot revive sessions.
            public["epoch"] = Value::from(
                public["epoch"]
                    .as_u64()
                    .unwrap_or(0)
                    .max(old["epoch"].as_u64().unwrap_or(0).saturating_add(1)),
            );
        }
        self.update_indexes(bucket, key, Some(&public))?;
        self.record_change(bucket, key, Some(public.clone()))?;
        self.import_record(bucket, key, &public)?;
        self.security_transition(bucket, key, before.as_ref(), Some(&public))
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
        let before = if matches!(bucket, "users" | "passkeys") {
            self.get::<Value>(bucket, key)?
        } else {
            None
        };
        self.update_indexes(bucket, key, None)?;
        self.record_change(bucket, key, None)?;
        self.raw_put(&record_key(bucket, key), None)?;
        self.security_transition(bucket, key, before.as_ref(), None)
    }

    pub(crate) fn mark_security_event(&self, user: &str, event: &str, credential: &str) -> bool {
        self.security_events
            .borrow_mut()
            .insert((user.into(), event.into(), credential.into()))
    }

    /// Security effects belong to the durable transition, irrespective of its API.
    /// Offline import_record deliberately bypasses this hook when restoring a snapshot.
    fn security_transition(
        &self,
        bucket: &str,
        key: &str,
        before: Option<&Value>,
        after: Option<&Value>,
    ) -> Result<()> {
        if bucket == "users" {
            if let Some(before) = before {
                crate::core::user_security_transition(self, key, before, after)?;
            }
        } else if bucket == "passkeys"
            && before.is_some() != after.is_some()
            && let Some(user) = after.or(before).and_then(|v| v["user_id"].as_str())
        {
            crate::ssf::enqueue(self, user, crate::ssf::CREDENTIAL_CHANGE, "public-key")?;
        }
        Ok(())
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
        if let Transaction::Prepared(store) = self.transaction {
            return store.read(|tx| tx.raw_scan_base(range));
        }
        let records = if let Transaction::Postgres(transaction, _) = &self.transaction {
            let limit = i64::try_from(range.limit).unwrap_or(i64::MAX);
            let query = if range.reverse {
                "SELECT key,value FROM riauth_store.records_v1 WHERE key >= $1 AND key < $2 ORDER BY key DESC LIMIT $3"
            } else {
                "SELECT key,value FROM riauth_store.records_v1 WHERE key >= $1 AND key < $2 ORDER BY key LIMIT $3"
            };
            transaction
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
                .collect::<Result<Vec<_>>>()?
        } else {
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
                .collect::<Result<Vec<_>>>()?
            })
        };
        self.telemetry
            .scanned_records
            .fetch_add(records.len() as u64, std::sync::atomic::Ordering::Relaxed);
        Ok(records)
    }
    /// One page of the whole keyspace, strictly after `after`.
    /// Repeat inside the same read transaction for a consistent scan.
    pub(crate) fn snapshot_page(
        &self,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, Value)>> {
        self.snapshot_page_raw(after, limit)?
            .into_iter()
            .map(|(name, bytes)| {
                let value = decode(self.key, &name, &bytes)?;
                Ok((name, value))
            })
            .collect()
    }
    fn snapshot_page_raw(
        &self,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, Vec<u8>)>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        if self.prepared.borrow().is_some() {
            return Err(Error::internal(
                "Paged snapshots require an ordinary read transaction",
            ));
        }
        if let Transaction::Postgres(transaction, _) = &self.transaction {
            let limit = i64::try_from(limit).unwrap_or(i64::MAX);
            let mut transaction = transaction.borrow_mut();
            let rows = if let Some(after) = after {
                transaction.query(
                    "SELECT key,value FROM riauth_store.records_v1 WHERE key > $1 ORDER BY key LIMIT $2",
                    &[&after.as_bytes(), &limit],
                )
            } else {
                transaction.query(
                    "SELECT key,value FROM riauth_store.records_v1 ORDER BY key LIMIT $1",
                    &[&limit],
                )
            }
            .map_err(crate::postgres_store::unavailable)?;
            return rows
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
            // `\0` is the inclusive successor used by bucket scans; it excludes `after`.
            let start = after.map(|key| format!("{key}\0")).unwrap_or_default();
            let iter = table.range(start.as_str()..).map_err(Error::internal)?;
            iter.take(limit)
                .map(|entry| {
                    let (key, value) = entry.map_err(Error::internal)?;
                    Ok((key.value().to_owned(), value.value().to_vec()))
                })
                .collect::<Result<Vec<_>>>()
        })
    }
    pub fn snapshot(&self) -> Result<BTreeMap<String, Value>> {
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
