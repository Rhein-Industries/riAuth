//! Optimistic preparation keeps hashing and remote signing outside the writer lock.
//! Point reads and range reads are validated again before any staged change commits.
use super::*;
use zeroize::Zeroize;

#[derive(Clone)]
pub(super) struct Range {
    pub start: String,
    pub end: String,
    pub limit: usize,
    pub reverse: bool,
}
struct ReadRange {
    range: Range,
    records: Vec<(String, Vec<u8>)>,
}
pub(super) struct Prepared {
    started_at: u64,
    deadline: Option<u64>,
    reads: BTreeMap<String, Option<Vec<u8>>>,
    writes: BTreeMap<String, Option<Vec<u8>>>,
    scans: Vec<ReadRange>,
}
impl Default for Prepared {
    fn default() -> Self {
        Self {
            started_at: crypto::now(),
            deadline: None,
            reads: BTreeMap::new(),
            writes: BTreeMap::new(),
            scans: Vec::new(),
        }
    }
}
impl Prepared {
    fn track_expiry(&mut self, key: Option<&[u8; 32]>, name: &str, bytes: &[u8]) -> Result<()> {
        let value: Value = decode(key, name, bytes)?;
        if let Some(expiry) = value["expires_at"]
            .as_u64()
            .filter(|expiry| *expiry > self.started_at)
        {
            self.deadline = Some(self.deadline.unwrap_or(u64::MAX).min(expiry));
        }
        Ok(())
    }
}
impl Drop for Prepared {
    fn drop(&mut self) {
        for bytes in self
            .reads
            .values_mut()
            .chain(self.writes.values_mut())
            .flatten()
        {
            bytes.zeroize();
        }
        for scan in &mut self.scans {
            for (_, bytes) in &mut scan.records {
                bytes.zeroize();
            }
        }
    }
}

impl Store {
    /// The callback may run again after a conflict. Do not perform irreversible
    /// external effects in it. Generated credentials are returned only after commit.
    pub fn prepared_write<T>(&self, mut f: impl FnMut(&Tx<'_>) -> Result<T>) -> Result<T> {
        for _ in 0..4 {
            // Each uncached read uses a short transaction. Expensive preparation
            // holds neither a writer nor a PostgreSQL pool slot. Point reads are
            // repeatable from the cache; all points/ranges must still match at commit.
            let tx = Tx {
                transaction: Transaction::Prepared(self),
                key: self.key(),
                changes: RefCell::default(),
                security_events: RefCell::default(),
                telemetry: &self.telemetry,
                prepared: RefCell::new(Some(Prepared::default())),
            };
            let output = f(&tx)?;
            let prepared = tx
                .prepared
                .borrow_mut()
                .take()
                .expect("prepared transaction");
            let committed = self.write(|tx| {
                // Authority can expire without a concurrent database mutation.
                if prepared
                    .deadline
                    .is_some_and(|deadline| crypto::now() >= deadline)
                {
                    return Ok(false);
                }
                for (name, expected) in &prepared.reads {
                    if tx.raw_get_base(name)? != *expected {
                        return Ok(false);
                    }
                }
                for scan in &prepared.scans {
                    if tx.raw_scan_base(&scan.range)? != scan.records {
                        return Ok(false);
                    }
                }
                for (name, bytes) in &prepared.writes {
                    tx.raw_put(name, bytes.clone())?;
                }
                Ok(true)
            })?;
            if committed {
                return Ok(output);
            }
            self.telemetry
                .optimistic_conflicts
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Err(Error::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "transaction_conflict",
            "Authentication state changed concurrently; retry the request",
        ))
    }
}

impl Tx<'_> {
    pub(super) fn raw_get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        if let Some(prepared) = self.prepared.borrow().as_ref() {
            if let Some(bytes) = prepared.writes.get(name) {
                return Ok(bytes.clone());
            }
            if let Some(bytes) = prepared.reads.get(name) {
                return Ok(bytes.clone());
            }
        }
        let bytes = self.raw_get_base(name)?;
        if let Some(prepared) = self.prepared.borrow_mut().as_mut() {
            if let Some(bytes) = &bytes {
                prepared.track_expiry(self.key, name, bytes)?;
            }
            prepared
                .reads
                .entry(name.into())
                .or_insert_with(|| bytes.clone());
        }
        Ok(bytes)
    }
    pub(super) fn stage(&self, name: &str, bytes: &Option<Vec<u8>>) -> Result<bool> {
        if self.prepared.borrow().is_none() {
            return Ok(false);
        }
        if !self
            .prepared
            .borrow()
            .as_ref()
            .unwrap()
            .reads
            .contains_key(name)
        {
            let before = self.raw_get_base(name)?;
            self.prepared
                .borrow_mut()
                .as_mut()
                .unwrap()
                .reads
                .insert(name.into(), before);
        }
        self.prepared
            .borrow_mut()
            .as_mut()
            .unwrap()
            .writes
            .insert(name.into(), bytes.clone());
        Ok(true)
    }
    pub(super) fn scan_records(&self, mut range: Range) -> Result<Vec<(String, Vec<u8>)>> {
        let limit = range.limit;
        if let Some(prepared) = self.prepared.borrow().as_ref() {
            // Staged deletions must not hide later records from a bounded scan.
            range.limit = range.limit.saturating_add(prepared.writes.len());
        }
        let records = self.raw_scan_base(&range)?;
        let mut prepared = self.prepared.borrow_mut();
        let Some(prepared) = prepared.as_mut() else {
            return Ok(records);
        };
        for (name, bytes) in &records {
            prepared.track_expiry(self.key, name, bytes)?;
        }
        prepared.scans.push(ReadRange {
            range: range.clone(),
            records: records.clone(),
        });
        let mut records: BTreeMap<_, _> = records.into_iter().collect();
        for (name, bytes) in prepared.writes.range(range.start..range.end) {
            match bytes {
                Some(bytes) => {
                    records.insert(name.clone(), bytes.clone());
                }
                None => {
                    records.remove(name);
                }
            }
        }
        Ok(if range.reverse {
            records.into_iter().rev().take(limit).collect()
        } else {
            records.into_iter().take(limit).collect()
        })
    }
}
