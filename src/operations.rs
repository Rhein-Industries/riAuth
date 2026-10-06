use crate::{
    config::{Config, private_dir, write_private},
    core::{Core, keys},
    crypto::{self, now},
    error::{Error, Result},
    model::{Client, User},
    store::Store,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

pub mod stream;

const BACKUP_V1: &str = "riauth.backup/v1";
const BACKUP_V2: &str = "riauth.backup/v2";
const MANIFEST_AAD: &[u8] = b"riauth.backup/v2/manifest";
/// Hard limit for the complete JSON archive, including legacy restore inputs.
pub const MAX_BACKUP_BYTES: usize = 64 * 1024 * 1024;
const MAX_BACKUP_PAGE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Deserialize)]
struct Backup {
    api_version: String,
    config: Config,
    records: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestChunk {
    kind: String,
    api_version: String,
    created_at: u64,
    config: Config,
    record_count: u64,
    digests: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordsChunk {
    kind: String,
    index: u64,
    records: BTreeMap<String, Value>,
}

impl Core {
    pub fn get_resource(&self, token: &str, kind: &str, name: &str) -> Result<Value> {
        self.store.read(|tx| match kind {
            "user" => {
                self.management(tx, token, "user.read", &format!("user/{name}"))?;
                Ok(json!(crate::model::UserView::from(
                    &crate::core::user_by_name(tx, name)?
                )))
            }
            "client" => {
                self.management(tx, token, "client.read", &format!("client/{name}"))?;
                tx.get::<Client>("clients", name)?
                    .map(|c| c.view())
                    .ok_or_else(|| Error::missing("Client not found"))
            }
            "group" => {
                self.management(tx, token, "group.read", &format!("group/{name}"))?;
                tx.get::<Value>("groups", name)?
                    .ok_or_else(|| Error::missing("Group not found"))
            }
            "source" => {
                self.management(tx, token, "source.read", &format!("source/{name}"))?;
                tx.get::<Value>("sources", name)?
                    .ok_or_else(|| Error::missing("Source not found"))
            }
            _ => Err(Error::bad("Unknown resource kind")),
        })
    }
    pub fn doctor(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            self.management(tx, token, "operations.read", "operations/health")?;
            let keys = keys(tx)?;
            keys.active.jwk()?;
            let (users, administrators) =
                count_bucket::<User>(tx, "users", |user| user.admin && user.enabled)?;
            let (clients, _) = count_bucket::<Client>(tx, "clients", |_| false)?;
            let (_, pending) =
                count_bucket::<crate::logout::Delivery>(tx, "logout_deliveries", |delivery| {
                    delivery.delivered_at.is_none()
                })?;
            let tls = if self.config.tls_cert_file.is_some() {
                "native_rustls"
            } else {
                "reverse_proxy"
            };
            Ok(json!({
                "healthy": administrators > 0,
                "schema_version": tx.get::<u32>("meta", "schema")?,
                "revision": tx.get::<u64>("meta", "revision")?.unwrap_or(0),
                "issuer": self.config.issuer,
                "storage": self.store.backend(),
                "encrypted_at_rest": self.config.database_key_file.is_some(),
                "active_signing_key": keys.active.kid,
                "users": users,
                "enabled_administrators": administrators,
                "clients": clients,
                "pending_logout_deliveries": pending,
                "tls": tls,
                "checked_at": now()
            }))
        })
    }
    pub fn backup(&self, token: &str, encryption_key: &str) -> Result<Value> {
        let key = decode_key(encryption_key)?;
        // Seal one maintenance page at a time. The manifest is sealed only after
        // the scan, so a truncated chunk list cannot authenticate as a full backup.
        // Callers receive one JSON document, bounded by MAX_BACKUP_BYTES.
        let (created_at, config, chunks, digests, record_count) = self.store.read(|tx| {
            self.management(tx, token, "operations.backup", "operations/backup")?;
            let mut config = self.config.clone();
            config.database_key_file = None;
            let created_at = now();
            let mut chunks = Vec::new();
            let mut digests = Vec::new();
            let mut record_count = 0u64;
            // Reserve space for the enclosing object and chunk separators.
            let mut archive_bytes = 512usize;
            let mut after: Option<String> = None;
            loop {
                let page = tx.snapshot_page_bounded(
                    after.as_deref(),
                    crate::store::maintenance::PAGE,
                    MAX_BACKUP_PAGE_BYTES,
                )?;
                if page.is_empty() {
                    break;
                }
                let next = page.last().unwrap().0.clone();
                if after
                    .as_ref()
                    .is_some_and(|previous| next.as_str() <= previous.as_str())
                {
                    return Err(Error::internal("Backup page did not advance"));
                }
                let len = page.len() as u64;
                let index = chunks.len();
                let body = RecordsChunk {
                    kind: "records".into(),
                    index: index as u64,
                    records: page.into_iter().collect(),
                };
                let plaintext = bounded_backup_json(&body, MAX_BACKUP_PAGE_BYTES)?;
                let ciphertext =
                    crypto::seal(&key, record_chunk_aad(index).as_bytes(), &plaintext)?;
                let encoded = URL_SAFE_NO_PAD.encode(ciphertext);
                archive_bytes += encoded.len() + 4;
                if archive_bytes > MAX_BACKUP_BYTES {
                    return Err(Error::bad("Backup archive exceeds 64 MiB"));
                }
                digests.push(crypto::digest(&encoded));
                chunks.push(encoded);
                record_count += len;
                after = Some(next);
            }
            Ok((created_at, config, chunks, digests, record_count))
        })?;
        let manifest = ManifestChunk {
            kind: "manifest".into(),
            api_version: BACKUP_V2.into(),
            created_at,
            config,
            record_count,
            digests,
        };
        let plaintext = bounded_backup_json(&manifest, MAX_BACKUP_PAGE_BYTES)?;
        let manifest_ciphertext = crypto::seal(&key, MANIFEST_AAD, &plaintext)?;
        let mut chunks = chunks;
        chunks.push(URL_SAFE_NO_PAD.encode(manifest_ciphertext));
        if 512 + chunks.iter().map(|chunk| chunk.len() + 4).sum::<usize>() > MAX_BACKUP_BYTES {
            return Err(Error::bad("Backup archive exceeds 64 MiB"));
        }
        let mut envelope = json!({
            "api_version": BACKUP_V2,
            "created_at": created_at,
            "encrypted": true,
        });
        envelope["chunks"] = Value::Array(chunks.into_iter().map(Value::String).collect());
        Ok(envelope)
    }
    pub fn inventory(
        &self,
        token: &str,
        kind: &str,
        after: Option<String>,
        limit: usize,
        filter: Option<String>,
    ) -> Result<Value> {
        let (bucket, action, resource_kind) = match kind {
            "users" => ("users", "user.read", "user"),
            "clients" => ("clients", "client.read", "client"),
            "groups" => ("groups", "group.read", "group"),
            "sources" => ("sources", "source.read", "source"),
            "audit" => ("audit", "audit.read", "audit"),
            _ => return Err(Error::bad("Unknown inventory kind")),
        };
        if after
            .as_ref()
            .is_some_and(|s| s.len() > 4096 || s.chars().any(char::is_control))
            || filter.as_ref().is_some_and(|s| s.len() > 256)
        {
            return Err(Error::bad("Invalid inventory cursor or filter"));
        }
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            if kind == "audit" { actor.require(action, "audit/events")?; }
            if actor.agent && !actor.permissions.iter().any(|p| p.action == action) { return Err(Error::forbidden()); }
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            use sha2::{Digest, Sha256};
            let key: [u8; 32] = Sha256::digest(tx.get::<String>("meta", "dummy_hash")?.ok_or_else(|| Error::internal("Missing cursor key material"))?.as_bytes()).into();
            let mut cursor = if let Some(encoded) = after {
                let raw = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| Error::bad("Invalid inventory cursor"))?;
                let plain = crypto::unseal(&key, b"riauth.inventory/v1", &raw).map_err(|_| Error::bad("Invalid inventory cursor"))?;
                let data: Value = serde_json::from_slice(&plain).map_err(|_| Error::bad("Invalid inventory cursor"))?;
                if data["actor"] != actor.id || data["kind"] != kind || data["filter"] != json!(filter) { return Err(Error::bad("Inventory cursor belongs to another query or actor")); }
                if data["revision"] != revision || data["expires_at"].as_u64().is_none_or(|at| at <= now()) { return Err(Error::conflict("Inventory changed or cursor expired; restart enumeration")); }
                Some(data["after"].as_str().ok_or_else(|| Error::bad("Invalid inventory cursor"))?.to_owned())
            } else { None };
            let limit = limit.clamp(1, 1000);
            let mut items = Vec::new(); let mut examined = 0;
            loop {
                let batch = tx.scan::<Value>(bucket, cursor.as_deref(), 256)?;
                if batch.is_empty() { cursor = None; break; }
                for (id, raw) in batch {
                    cursor = Some(id.clone()); examined += 1;
                    let (name, public) = match kind {
                        "users" => { let user: User = serde_json::from_value(raw).map_err(Error::internal)?; (user.username.clone(), json!(crate::model::UserView::from(&user))) }
                        "clients" => { let client: Client = serde_json::from_value(raw).map_err(Error::internal)?; (client.id.clone(), client.view()) }
                        "groups" => (raw["name"].as_str().unwrap_or("").to_owned(), raw),
                        "sources" => (raw["id"].as_str().unwrap_or("").to_owned(), raw),
                        _ => ("events".into(), raw),
                    };
                    if actor.allows(action, &format!("{resource_kind}/{name}")) && filter.as_ref().is_none_or(|f| if kind == "audit" { public["run_id"].as_str() == Some(f) } else { name.contains(f) }) { items.push(public); }
                    if items.len() == limit || examined >= 10_000 { break; }
                }
                if items.len() == limit || examined >= 10_000 { break; }
            }
            let cursor = cursor.map(|after| -> Result<String> {
                let value = json!({"actor": actor.id, "kind": kind, "filter": filter, "revision": revision, "expires_at": now() + 3600, "after": after});
                Ok(URL_SAFE_NO_PAD.encode(crypto::seal(&key, b"riauth.inventory/v1", &serde_json::to_vec(&value).map_err(Error::internal)?)?))
            }).transpose()?;
            Ok(json!({"items": items, "next_cursor": cursor, "limit": limit, "revision": revision}))
        })
    }
}

/// Count one bucket inside the caller's snapshot. Each page is released
/// before the next read. A stored value is still decoded in full; this path
/// does not size-cap it. An exact multiple of the page size reads one extra
/// empty page. A cursor that does not advance is an internal error.
fn count_bucket<T: DeserializeOwned>(
    tx: &crate::store::Tx<'_>,
    bucket: &str,
    mut matches: impl FnMut(&T) -> bool,
) -> Result<(u64, u64)> {
    let mut after: Option<String> = None;
    let mut total = 0u64;
    let mut matched = 0u64;
    loop {
        let page = tx.scan::<T>(bucket, after.as_deref(), crate::store::maintenance::PAGE)?;
        let Some((last_key, _)) = page.last() else {
            break;
        };
        let last_key = last_key.clone();
        if after
            .as_ref()
            .is_some_and(|previous| last_key.as_str() <= previous.as_str())
        {
            return Err(Error::internal("Doctor page did not advance"));
        }
        let full = page.len() == crate::store::maintenance::PAGE;
        after = Some(last_key);
        for (_, row) in &page {
            if matches(row) {
                matched = matched
                    .checked_add(1)
                    .ok_or_else(|| Error::internal("Doctor count overflowed"))?;
            }
        }
        total = total
            .checked_add(u64::try_from(page.len()).map_err(Error::internal)?)
            .ok_or_else(|| Error::internal("Doctor count overflowed"))?;
        if !full {
            break;
        }
    }
    Ok((total, matched))
}

fn bounded_backup_json(
    value: &impl Serialize,
    limit: usize,
) -> Result<zeroize::Zeroizing<Vec<u8>>> {
    struct LimitedBuffer {
        bytes: zeroize::Zeroizing<Vec<u8>>,
        limit: usize,
    }
    impl Write for LimitedBuffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
                return Err(std::io::Error::other("Backup plaintext page exceeds 8 MiB"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = LimitedBuffer {
        bytes: zeroize::Zeroizing::new(Vec::new()),
        limit,
    };
    serde_json::to_writer(&mut buffer, value)
        .map_err(|_| Error::bad("Backup plaintext page exceeds 8 MiB"))?;
    Ok(buffer.bytes)
}

fn decode_key(encoded: &str) -> Result<zeroize::Zeroizing<[u8; 32]>> {
    let bytes = zeroize::Zeroizing::new(
        URL_SAFE_NO_PAD
            .decode(encoded.trim())
            .map_err(|_| Error::bad("Invalid backup key encoding"))?,
    );
    Ok(zeroize::Zeroizing::new(
        bytes
            .as_slice()
            .try_into()
            .map_err(|_| Error::bad("Backup key must contain 32 bytes"))?,
    ))
}

/// The offline destination for an authenticated backup. PostgreSQL must be a
/// dedicated, empty database; restore never replaces records in an existing one.
#[derive(Clone)]
pub enum RestoreTarget {
    Redb,
    Postgres(crate::postgres_store::PostgresConfig),
}

pub fn restore(
    backup_file: &Path,
    key_file: &Path,
    output: &Path,
    database_key_file: Option<PathBuf>,
) -> Result<Value> {
    restore_into(
        backup_file,
        key_file,
        output,
        database_key_file,
        RestoreTarget::Redb,
    )
}

pub fn restore_into(
    backup_file: &Path,
    key_file: &Path,
    output: &Path,
    database_key_file: Option<PathBuf>,
    target: RestoreTarget,
) -> Result<Value> {
    if stream::is_stream_archive(backup_file)? {
        return stream::restore_stream_into(
            backup_file,
            key_file,
            output,
            database_key_file,
            target,
            stream::StreamOptions::default(),
        );
    }
    let file = std::fs::File::open(backup_file).map_err(Error::internal)?;
    if file.metadata().map_err(Error::internal)?.len() > MAX_BACKUP_BYTES as u64 {
        return Err(Error::bad("Backup archive exceeds 64 MiB"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_BACKUP_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(Error::internal)?;
    if bytes.len() > MAX_BACKUP_BYTES {
        return Err(Error::bad("Backup archive exceeds 64 MiB"));
    }
    let envelope: Value = serde_json::from_slice(&bytes).map_err(Error::internal)?;
    drop(bytes);
    let version = envelope["api_version"].as_str().unwrap_or("");
    if version != BACKUP_V1 && version != BACKUP_V2 {
        return Err(Error::bad("Unsupported backup format"));
    }
    let key = crypto::read_key(key_file)?;
    match version {
        BACKUP_V1 => restore_v1(&envelope, &key, output, database_key_file, target),
        _ => restore_v2(&envelope, &key, output, database_key_file, target),
    }
}

fn restore_v1(
    envelope: &Value,
    key: &[u8; 32],
    output: &Path,
    database_key_file: Option<PathBuf>,
    target: RestoreTarget,
) -> Result<Value> {
    let ciphertext = URL_SAFE_NO_PAD
        .decode(
            envelope["ciphertext"]
                .as_str()
                .ok_or_else(|| Error::bad("Missing backup ciphertext"))?,
        )
        .map_err(|_| Error::bad("Invalid backup ciphertext"))?;
    let plaintext = crypto::unseal(key, BACKUP_V1.as_bytes(), &ciphertext)?;
    let backup: Backup =
        serde_json::from_slice(&plaintext).map_err(|_| Error::bad("Backup payload is invalid"))?;
    if backup.api_version != BACKUP_V1 || !schema_supported(backup.records.get("meta/schema")) {
        return Err(Error::bad("Unsupported backup schema"));
    }
    if backup.records.get("meta/issuer") != Some(&json!(backup.config.issuer)) {
        return Err(Error::bad("Backup issuer mismatch"));
    }
    let records = backup.records;
    commit_restore(
        backup.config,
        // v1 does not authenticate its creation time.
        None,
        output,
        database_key_file,
        target,
        move |tx| {
            for (name, value) in records {
                let (bucket, id) = split_record_key(&name)?;
                tx.import_snapshot_record(bucket, id, &value)?;
            }
            Ok(())
        },
        || Ok(()),
    )
}

struct ChunkedBackup {
    config: Config,
    created_at: u64,
    record_count: u64,
}

fn restore_v2(
    envelope: &Value,
    key: &[u8; 32],
    output: &Path,
    database_key_file: Option<PathBuf>,
    target: RestoreTarget,
) -> Result<Value> {
    if envelope["encrypted"] != true {
        return Err(Error::bad("Unsupported backup format"));
    }
    let loaded = load_chunked(envelope, key)?;
    let chunks = envelope["chunks"].as_array().unwrap();
    let record_count = loaded.record_count;
    let key = *key;
    commit_restore(
        loaded.config,
        Some(loaded.created_at),
        output,
        database_key_file,
        target,
        move |tx| {
            let mut imported = 0u64;
            for (index, encoded) in chunks[..chunks.len() - 1].iter().enumerate() {
                let ciphertext = URL_SAFE_NO_PAD
                    .decode(encoded.as_str().unwrap())
                    .map_err(|_| Error::bad("Invalid backup ciphertext"))?;
                let plaintext =
                    crypto::unseal(&key, record_chunk_aad(index).as_bytes(), &ciphertext)?;
                let chunk: RecordsChunk = serde_json::from_slice(&plaintext)
                    .map_err(|_| Error::bad("Backup payload is invalid"))?;
                if chunk.kind != "records" || chunk.index != index as u64 {
                    return Err(Error::bad("Backup payload is invalid"));
                }
                for (name, value) in chunk.records {
                    let (bucket, id) = split_record_key(&name)?;
                    if tx.get::<Value>(bucket, id)?.is_some() {
                        return Err(Error::bad("Backup payload is invalid"));
                    }
                    tx.import_snapshot_record(bucket, id, &value)?;
                    imported += 1;
                }
            }
            if imported != record_count {
                return Err(Error::bad("Backup payload is invalid"));
            }
            Ok(())
        },
        || Ok(()),
    )
}

/// Authenticate every chunk before creating a directory. Plaintext pages are dropped.
fn load_chunked(envelope: &Value, key: &[u8; 32]) -> Result<ChunkedBackup> {
    let listed = envelope["chunks"]
        .as_array()
        .ok_or_else(|| Error::bad("Unsupported backup format"))?;
    if listed.is_empty() {
        return Err(Error::bad("Unsupported backup format"));
    }
    let (manifest_encoded, record_encoded) = listed.split_last().unwrap();
    let manifest_encoded = manifest_encoded
        .as_str()
        .ok_or_else(|| Error::bad("Invalid backup ciphertext"))?;
    let manifest_bytes = URL_SAFE_NO_PAD
        .decode(manifest_encoded)
        .map_err(|_| Error::bad("Invalid backup ciphertext"))?;
    let manifest_plain = crypto::unseal(key, MANIFEST_AAD, &manifest_bytes)?;
    let manifest: ManifestChunk = serde_json::from_slice(&manifest_plain)
        .map_err(|_| Error::bad("Backup payload is invalid"))?;
    if manifest.kind != "manifest" || manifest.api_version != BACKUP_V2 {
        return Err(Error::bad("Unsupported backup format"));
    }
    if envelope["created_at"].as_u64() != Some(manifest.created_at)
        || manifest.digests.len() != record_encoded.len()
    {
        return Err(Error::bad("Backup payload is invalid"));
    }
    let mut schema = None;
    let mut issuer = None;
    let mut count = 0u64;
    for (index, (encoded, digest)) in record_encoded.iter().zip(&manifest.digests).enumerate() {
        let encoded = encoded
            .as_str()
            .ok_or_else(|| Error::bad("Invalid backup ciphertext"))?;
        if crypto::digest(encoded) != *digest {
            return Err(Error::bad("Backup payload is invalid"));
        }
        let ciphertext = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| Error::bad("Invalid backup ciphertext"))?;
        let plaintext = crypto::unseal(key, record_chunk_aad(index).as_bytes(), &ciphertext)?;
        let chunk: RecordsChunk = serde_json::from_slice(&plaintext)
            .map_err(|_| Error::bad("Backup payload is invalid"))?;
        if chunk.kind != "records" || chunk.index != index as u64 {
            return Err(Error::bad("Backup payload is invalid"));
        }
        for (name, value) in &chunk.records {
            split_record_key(name)?;
            count += 1;
            if name == "meta/schema" {
                schema = Some(value.clone());
            }
            if name == "meta/issuer" {
                issuer = Some(value.clone());
            }
        }
    }
    if count != manifest.record_count || !schema_supported(schema.as_ref()) {
        return Err(Error::bad("Unsupported backup schema"));
    }
    if issuer.as_ref() != Some(&json!(manifest.config.issuer)) {
        return Err(Error::bad("Backup issuer mismatch"));
    }
    Ok(ChunkedBackup {
        config: manifest.config,
        created_at: manifest.created_at,
        record_count: manifest.record_count,
    })
}

/// Every restore format must call this: it applies the restored-state policy
/// (`crate::recovery`) in the same transaction as the import.
fn commit_restore(
    mut config: Config,
    snapshot_created_at: Option<u64>,
    output: &Path,
    database_key_file: Option<PathBuf>,
    target: RestoreTarget,
    import: impl FnOnce(&crate::store::Tx<'_>) -> Result<()>,
    check: impl Fn() -> Result<()>,
) -> Result<Value> {
    let database_key_file = database_key_file
        .map(|p| p.canonicalize().map_err(Error::internal))
        .transpose()?;
    let storage_key = database_key_file
        .as_deref()
        .map(crypto::read_key)
        .transpose()?;
    config.data_dir = "data".into();
    config.postgres = match target {
        RestoreTarget::Redb => None,
        RestoreTarget::Postgres(mut postgres) => {
            // Public API callers may pass relative files. Persist absolute
            // references so the output config opens the database we inspected.
            postgres.connection_file = postgres
                .connection_file
                .canonicalize()
                .map_err(Error::internal)?;
            postgres.ca_file = postgres
                .ca_file
                .map(|path| path.canonicalize().map_err(Error::internal))
                .transpose()?;
            Some(postgres)
        }
    };
    config.database_key_file = database_key_file;
    config
        .validate()
        .map_err(|_| Error::bad("Invalid backup configuration"))?;
    let config_bytes = toml::to_string_pretty(&config).map_err(Error::internal)?;
    check()?;
    if config.postgres.is_some() {
        // Read-only preflight keeps an occupied target and the output directory
        // untouched. The writer repeats this check under PostgreSQL's lock.
        Store::inspect(&config, |_, tx| {
            if tx
                .map(|tx| tx.snapshot_next_key(None))
                .transpose()?
                .flatten()
                .is_some()
            {
                return Err(Error::conflict(
                    "Restore requires an empty PostgreSQL target",
                ));
            }
            Ok(())
        })?;
    }
    std::fs::create_dir(output).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            Error::conflict("Restore requires a new output directory")
        } else {
            Error::bad(format!("Restore requires a new output directory: {error}"))
        }
    })?;
    private_dir(output).map_err(Error::internal)?;
    let store = if let Some(postgres) = &config.postgres {
        Store::open_postgres(postgres.clone(), storage_key)?
    } else {
        let data_dir = output.join("data");
        private_dir(&data_dir).map_err(Error::internal)?;
        Store::open_with_key(&data_dir.join("riauth.redb"), storage_key)?
    };
    let recovery = store.write(|tx| {
        check()?;
        if config.postgres.is_some() {
            if tx.snapshot_next_key(None)?.is_some() {
                return Err(Error::conflict(
                    "Restore requires an empty PostgreSQL target",
                ));
            }
            if tx.postgres_other_clients()?.is_some_and(|count| count > 0) {
                return Err(Error::conflict(
                    "Stop every riAuth process connected to the PostgreSQL target before restore",
                ));
            }
        }
        import(tx)?;
        check()?;
        // Invalidation and the serving gate must commit with the imported state.
        let recovery = crate::recovery::invalidate(
            tx,
            crate::recovery::Cause::BackupRestore,
            snapshot_created_at,
            None,
        )?;
        check()?;
        tx.rebuild_indexes_checked(&check)?;
        check()?;
        if tx.get::<String>("meta", "issuer")?.as_deref() != Some(&config.issuer)
            || !schema_supported(tx.get::<Value>("meta", "schema")?.as_ref())
            || tx.get::<String>("meta", "dummy_hash")?.is_none()
        {
            return Err(Error::bad("Backup state is incomplete"));
        }
        crate::keyring::public_keys(tx)?;
        let mut after = None;
        let mut enabled_administrator = false;
        loop {
            check()?;
            // Imported users can approach the frame limit; one decoded record
            // is the predictable bound for this validation loop.
            let users = tx.scan::<User>("users", after.as_deref(), 1)?;
            if users.is_empty() {
                break;
            }
            for (id, user) in users {
                check()?;
                if user.admin && user.enabled {
                    enabled_administrator |=
                        tx.get::<String>("usernames", &user.username)? == Some(user.id.clone());
                }
                crate::claims::validate_user_checked(tx, &user, &check)?;
                after = Some(id);
            }
        }
        if !enabled_administrator {
            return Err(Error::bad("Backup has no enabled administrator"));
        }
        let mut after = None;
        loop {
            check()?;
            let clients = tx.scan::<Client>("clients", after.as_deref(), 1)?;
            let Some((id, client)) = clients.into_iter().next() else {
                break;
            };
            crate::core::validate_client(tx, &client)?;
            after = Some(id);
        }
        // A copied PostgreSQL lineage belongs to the source. Stamp a new
        // PostgreSQL target, or remove it when the target is redb.
        if config.postgres.is_some() {
            crate::recovery::stamp_lineage(tx)?;
        } else {
            tx.delete("meta", "storage_lineage")?;
        }
        check()?;
        Ok(recovery)
    })?;
    drop(store);
    // Reopen from the exact configuration that will be published. A failed
    // schema upgrade or key check leaves only a private candidate, not the
    // configuration an operator would use to activate the restored instance.
    let candidate_path = output.join(".riauth.restore-pending.toml");
    write_private(&candidate_path, config_bytes.as_bytes(), false).map_err(Error::internal)?;
    let config = Config::load(&candidate_path).map_err(Error::internal)?;
    let core = Core::open(config)?;
    core.jwks()?;
    if core.store.get::<u32>("meta", "schema")? != Some(crate::upgrade::SCHEMA)
        || core
            .store
            .read(crate::recovery::pending)?
            .as_ref()
            .map(|pending| pending.id.as_str())
            != Some(recovery.id.as_str())
    {
        return Err(Error::internal("Restored storage verification failed"));
    }
    let config_path = output.join("riauth.toml");
    write_private(&config_path, config_bytes.as_bytes(), false).map_err(Error::internal)?;
    let _ = std::fs::remove_file(candidate_path);
    Ok(
        json!({"restored": true, "verified": true, "config": config_path, "issuer": core.config.issuer, "encrypted_at_rest": core.config.database_key_file.is_some(),
            "storage": core.store.backend(),
            "serving_allowed": false, "recovery": recovery,
            "next": "Reconcile or rotate the listed persistent credentials, then run `riauth recovery complete --recovery-id <recovery.id> --persistent-credentials-reconciled`"}),
    )
}

fn schema_supported(schema: Option<&Value>) -> bool {
    schema
        .and_then(Value::as_u64)
        .is_some_and(|s| (1..=u64::from(crate::upgrade::SCHEMA)).contains(&s))
}

fn split_record_key(name: &str) -> Result<(&str, &str)> {
    name.split_once('/')
        .filter(|(bucket, key)| !bucket.is_empty() && !key.is_empty())
        .ok_or_else(|| Error::bad("Invalid backup record key"))
}

fn record_chunk_aad(index: usize) -> String {
    format!("riauth.backup/v2/records/{index}")
}

pub fn migrate_postgres(
    config: Config,
    target: crate::postgres_store::PostgresConfig,
    output: &Path,
) -> Result<Value> {
    migrate_postgres_checkpointed(config, target, output, |_| Ok(()))
}

/// A target prefix is the checkpoint: each page commits atomically, and a retry
/// compares that prefix with the exclusively opened source before appending.
/// The callback runs only after a page commit (also used by the interruption
/// regression); production has no callback side effects.
fn migrate_postgres_checkpointed(
    config: Config,
    mut target: crate::postgres_store::PostgresConfig,
    output: &Path,
    mut checkpoint: impl FnMut(u64) -> Result<()>,
) -> Result<Value> {
    if output.exists() {
        return Err(Error::conflict("Output configuration already exists"));
    }
    if config.postgres.is_some() {
        return Err(Error::bad("This migration requires an offline redb source"));
    }
    // The published file must reopen the same target that was verified, even
    // when a caller supplied paths relative to its current directory.
    target.connection_file = target
        .connection_file
        .canonicalize()
        .map_err(Error::internal)?;
    target.ca_file = target
        .ca_file
        .map(|file| file.canonicalize().map_err(Error::internal))
        .transpose()?;
    let mut migrated_config = config.clone();
    migrated_config.postgres = Some(target.clone());
    migrated_config.validate().map_err(Error::internal)?;
    let config_bytes = toml::to_string_pretty(&migrated_config).map_err(Error::internal)?;
    let core = Core::open(config.clone())?;
    core.jwks()?;
    let target_key = config
        .database_key_file
        .as_deref()
        .map(crypto::read_key)
        .transpose()?;
    let store = Store::open_postgres(target, target_key)?;
    let count = core.store.read(|source| {
        let (mut after, mut copied) = store.read(|destination| {
            require_isolated_migration_target(destination)?;
            let prefix = compare_migration_records(source, destination, false, false)?;
            let _ = destination.get::<crate::recovery::Lineage>("meta", "storage_lineage")?;
            Ok(prefix)
        })?;
        loop {
            let page = source.snapshot_page_bounded(
                after.as_deref(),
                crate::store::maintenance::PAGE,
                MAX_BACKUP_PAGE_BYTES,
            )?;
            let Some((next, _)) = page.last() else {
                break;
            };
            let next = next.clone();
            if after
                .as_ref()
                .is_some_and(|previous| next.as_str() <= previous.as_str())
            {
                return Err(Error::internal("Migration page did not advance"));
            }
            store.write(|destination| {
                require_isolated_migration_target(destination)?;
                for (name, value) in &page {
                    let (bucket, key) = split_record_key(name)?;
                    match destination.get::<Value>(bucket, key)? {
                        Some(existing) if existing == *value => {}
                        Some(_) => {
                            return Err(Error::conflict(
                                "Target database does not match this migration",
                            ));
                        }
                        None => destination.import_snapshot_record(bucket, key, value)?,
                    }
                }
                Ok(())
            })?;
            copied += page.len() as u64;
            after = Some(next);
            checkpoint(copied)?;
        }
        // The PostgreSQL table lock fences direct SQL writers as well as the
        // ordinary riAuth writer lock for this complete, bounded comparison.
        store.write(|destination| {
            require_isolated_migration_target(destination)?;
            destination.lock_records_for_transition()?;
            let (_, verified) = compare_migration_records(source, destination, true, true)?;
            let _ = destination.get::<crate::recovery::Lineage>("meta", "storage_lineage")?;
            if verified != copied {
                return Err(Error::internal("Migrated storage verification failed"));
            }
            Ok(())
        })?;
        Ok(copied)
    })?;
    // Keep PostgreSQL lineage out of the copy. On first Core open, its normal
    // lineage gate stamps a new target or applies recovery to a changed one.
    write_private(output, config_bytes.as_bytes(), false).map_err(Error::internal)?;
    Ok(
        json!({"migrated":true,"records":count,"config":output,"issuer":config.issuer,"source_preserved":true}),
    )
}

fn require_isolated_migration_target(tx: &crate::store::Tx<'_>) -> Result<()> {
    if tx.postgres_other_clients()?.is_some_and(|count| count > 0) {
        return Err(Error::conflict(
            "Stop every riAuth process connected to the PostgreSQL target before migration",
        ));
    }
    Ok(())
}

struct MigrationCursor<'tx, 'db> {
    tx: &'tx crate::store::Tx<'db>,
    after: Option<String>,
    page: std::vec::IntoIter<(String, Value)>,
    locked_writer: bool,
    skip_lineage: bool,
}

impl MigrationCursor<'_, '_> {
    fn next(&mut self) -> Result<Option<(String, Value)>> {
        loop {
            if let Some((name, value)) = self.page.next() {
                if self.skip_lineage && name == "meta/storage_lineage" {
                    continue;
                }
                return Ok(Some((name, value)));
            }
            let page = if self.locked_writer {
                self.tx.snapshot_page_bounded_locked_writer(
                    self.after.as_deref(),
                    crate::store::maintenance::PAGE,
                    MAX_BACKUP_PAGE_BYTES,
                )?
            } else {
                self.tx.snapshot_page_bounded(
                    self.after.as_deref(),
                    crate::store::maintenance::PAGE,
                    MAX_BACKUP_PAGE_BYTES,
                )?
            };
            let Some((last, _)) = page.last() else {
                return Ok(None);
            };
            if self.after.as_ref().is_some_and(|previous| last <= previous) {
                return Err(Error::internal("Migration page did not advance"));
            }
            self.after = Some(last.clone());
            self.page = page.into_iter();
        }
    }
}

/// Return the last matching source key and count. Existing target records must
/// form a prefix of the source (apart from backend-specific lineage).
fn compare_migration_records(
    source: &crate::store::Tx<'_>,
    destination: &crate::store::Tx<'_>,
    complete: bool,
    locked_writer: bool,
) -> Result<(Option<String>, u64)> {
    let mut source = MigrationCursor {
        tx: source,
        after: None,
        page: Vec::new().into_iter(),
        locked_writer: false,
        skip_lineage: false,
    };
    let mut destination = MigrationCursor {
        tx: destination,
        after: None,
        page: Vec::new().into_iter(),
        locked_writer,
        skip_lineage: true,
    };
    let mut last = None;
    let mut count = 0u64;
    while let Some((target_name, target_value)) = destination.next()? {
        let Some((source_name, source_value)) = source.next()? else {
            return Err(Error::conflict(
                "Target database does not match this migration",
            ));
        };
        split_record_key(&source_name)?;
        if target_name != source_name || target_value != source_value {
            return Err(Error::conflict(
                "Target database does not match this migration",
            ));
        }
        last = Some(source_name);
        count += 1;
    }
    if complete && source.next()?.is_some() {
        return Err(Error::conflict("Migrated storage verification failed"));
    }
    Ok((last, count))
}

struct AlertPost {
    url: String,
    payload: Value,
    bearer: Option<zeroize::Zeroizing<String>>,
}

enum AlertPrep {
    Quiet,
    Failed { signals: Value },
    Post(AlertPost),
}

/// POST a small JSON summary when a selected condition is true.
/// Delivery failure is logged and counted. It does not stop the caller.
pub async fn dispatch_alerts(core: &Core) -> Result<Value> {
    if core.config.alert_webhook.is_none() {
        return Ok(json!({"delivered": false, "signals": []}));
    }
    let probe = core.clone();
    let prepared = tokio::task::spawn_blocking(move || {
        crate::telemetry::in_activity(crate::telemetry::Activity::Maintenance, || {
            prepare_alert(&probe)
        })
    })
    .await
    .map_err(Error::internal)?;
    match prepared {
        AlertPrep::Quiet => Ok(json!({"delivered": false, "signals": []})),
        AlertPrep::Failed { signals } => {
            Ok(json!({"delivered": false, "delivery_error": true, "signals": signals}))
        }
        AlertPrep::Post(post) => post_alert(core, post).await,
    }
}

fn prepare_alert(core: &Core) -> AlertPrep {
    let Some(webhook) = &core.config.alert_webhook else {
        return AlertPrep::Quiet;
    };
    if webhook.validate().is_err() {
        note_alert_failure(core, "invalid_webhook");
        return AlertPrep::Failed { signals: json!([]) };
    }
    let mut signals = Vec::new();
    if webhook.signals.storage_not_ready && core.store.ready().is_err() {
        signals.push(json!({"name": "storage_not_ready"}));
    }
    if webhook.signals.cleanup_errors {
        let count = core
            .store
            .telemetry()
            .cleanup_errors
            .load(std::sync::atomic::Ordering::Relaxed);
        if count > 0 {
            signals.push(json!({"name": "cleanup_errors", "count": count}));
        }
    }
    if webhook.signals.signing_failures {
        let count = core
            .store
            .telemetry()
            .signing_errors
            .load(std::sync::atomic::Ordering::Relaxed);
        if count > 0 {
            signals.push(json!({"name": "signing_failures", "count": count}));
        }
    }
    if signals.is_empty() {
        return AlertPrep::Quiet;
    }
    let signals = json!(signals);
    let bearer = if let Some(path) = &webhook.bearer_file {
        match bearer_token(path) {
            Ok(token) => Some(token),
            Err(()) => {
                note_alert_failure(core, "credential_unreadable");
                return AlertPrep::Failed { signals };
            }
        }
    } else {
        None
    };
    AlertPrep::Post(AlertPost {
        url: webhook.url.clone(),
        payload: json!({"service": "riAuth", "signals": signals}),
        bearer,
    })
}

fn bearer_token(path: &Path) -> std::result::Result<zeroize::Zeroizing<String>, ()> {
    let mut token = crate::config::read_private_secret(path, 4096).map_err(|_| ())?;
    while token.ends_with(['\n', '\r', ' ', '\t']) {
        token.pop();
    }
    if token.is_empty()
        || token.len() > 4096
        || token.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return Err(());
    }
    Ok(token)
}

async fn post_alert(core: &Core, post: AlertPost) -> Result<Value> {
    let signals = post.payload["signals"].clone();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .connect_timeout(Duration::from_secs(2))
        .redirect(reqwest::redirect::Policy::none())
        .build();
    let client = match client {
        Ok(client) => client,
        Err(_) => {
            note_alert_failure(core, "request_failed");
            return Ok(json!({"delivered": false, "delivery_error": true, "signals": signals}));
        }
    };
    let mut request = client.post(&post.url).json(&post.payload);
    if let Some(token) = &post.bearer {
        request = request.bearer_auth(token.as_str());
    }
    match request.send().await {
        Ok(response) if response.status().is_success() => {
            Ok(json!({"delivered": true, "signals": signals}))
        }
        Ok(_) => {
            note_alert_failure(core, "rejected");
            Ok(json!({"delivered": false, "delivery_error": true, "signals": signals}))
        }
        Err(_) => {
            note_alert_failure(core, "request_failed");
            Ok(json!({"delivered": false, "delivery_error": true, "signals": signals}))
        }
    }
}

fn note_alert_failure(core: &Core, reason: &'static str) {
    core.store
        .telemetry()
        .alert_delivery_errors
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    tracing::warn!(reason, "alert webhook delivery failed");
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    use crate::{model::NewUser, postgres_store::PostgresConfig};

    #[test]
    #[ignore = "requires a disposable PostgreSQL cluster in RIAUTH_TEST_PG_CONNECTION"]
    fn interrupted_postgres_migration_checks_its_checkpoint_before_resuming() {
        let shared = std::fs::read_to_string(
            std::env::var_os("RIAUTH_TEST_PG_CONNECTION")
                .expect("set RIAUTH_TEST_PG_CONNECTION to a disposable cluster"),
        )
        .unwrap();
        let database = format!("riauth_s06_{}", uuid::Uuid::new_v4().simple());
        let mut admin = postgres::Client::connect(shared.trim(), postgres::NoTls).unwrap();
        admin
            .batch_execute(&format!("CREATE DATABASE {database}"))
            .unwrap();
        let local = tempfile::tempdir().unwrap();
        let connection_file = local.path().join("connection");
        let connection = shared.replace("dbname=postgres", &format!("dbname={database}"));
        assert_ne!(connection, shared);
        write_private(&connection_file, connection.as_bytes(), false).unwrap();
        let postgres = PostgresConfig {
            connection_file,
            ca_file: None,
            local_unencrypted: true,
            pool_size: 2,
        };
        let config = Config {
            data_dir: local.path().join("data"),
            ..Default::default()
        };
        let source = Core::initialize(
            config.clone(),
            NewUser {
                username: "admin".into(),
                password: "migration-interruption-password".into(),
                email: None,
                display_name: "Admin".into(),
                admin: true,
            },
        )
        .unwrap();
        let token = source
            .login(
                "admin".into(),
                "migration-interruption-password".into(),
                None,
            )
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        source
            .store
            .write(|tx| {
                for index in 0..300 {
                    tx.put(
                        "audit",
                        &format!("migration-{index:04}"),
                        &json!({"index":index}),
                    )?;
                }
                Ok(())
            })
            .unwrap();
        let before = source.store.read(|tx| tx.snapshot()).unwrap();
        drop(source);

        let output = local.path().join("postgres.toml");
        let error =
            migrate_postgres_checkpointed(config.clone(), postgres.clone(), &output, |_| {
                Err(Error::conflict(
                    "simulated interruption after a committed page",
                ))
            })
            .unwrap_err();
        assert!(error.message.contains("simulated interruption"));
        assert!(!output.exists());
        let source = Core::open(config.clone()).unwrap();
        assert_eq!(source.store.read(|tx| tx.snapshot()).unwrap(), before);
        drop(source);
        let target = Store::open_postgres(postgres.clone(), None).unwrap();
        let partial = target.read(|tx| tx.snapshot()).unwrap();
        assert!(partial.len() >= crate::store::maintenance::PAGE);
        assert!(partial.len() < before.len());
        drop(target);

        let first = before.keys().next().unwrap();
        let mut raw = postgres::Client::connect(connection.trim(), postgres::NoTls).unwrap();
        raw.execute(
            "UPDATE riauth_store.records_v1 SET value=$2 WHERE key=$1",
            &[&first.as_bytes(), &b"null".as_slice()],
        )
        .unwrap();
        let error = migrate_postgres(config.clone(), postgres.clone(), &output).unwrap_err();
        assert!(error.message.contains("does not match this migration"));
        assert!(!output.exists());
        let value: Vec<u8> = raw
            .query_one(
                "SELECT value FROM riauth_store.records_v1 WHERE key=$1",
                &[&first.as_bytes()],
            )
            .unwrap()
            .get(0);
        assert_eq!(value, b"null");
        raw.execute(
            "UPDATE riauth_store.records_v1 SET value=$2 WHERE key=$1",
            &[
                &first.as_bytes(),
                &serde_json::to_vec(&before[first]).unwrap(),
            ],
        )
        .unwrap();

        let result = migrate_postgres(config.clone(), postgres.clone(), &output).unwrap();
        assert_eq!(result["migrated"], true);
        assert_eq!(result["records"], before.len());
        let target = Core::open(Config::load(&output).unwrap()).unwrap();
        let mut after = target.store.read(|tx| tx.snapshot()).unwrap();
        after.remove("meta/storage_lineage");
        assert_eq!(after, before);
        assert_eq!(target.me(&token).unwrap()["user"]["username"], "admin");
        drop(target);
        let source = Core::open(config).unwrap();
        assert_eq!(source.store.read(|tx| tx.snapshot()).unwrap(), before);
    }
}
