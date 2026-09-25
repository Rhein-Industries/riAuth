//! Bounded password-hash history. Plaintext passwords are never stored.
use crate::{
    crypto,
    error::{Error, Result},
    store::Tx,
};
use std::collections::BTreeSet;

const BUCKET: &str = "password_history";

fn load(tx: &Tx<'_>, user_id: &str) -> Result<Vec<String>> {
    let mut history = tx.get::<Vec<String>>(BUCKET, user_id)?.unwrap_or_default();
    history.retain(|hash| !hash.is_empty());
    Ok(history)
}

fn save(tx: &Tx<'_>, user_id: &str, history: &[String]) -> Result<()> {
    if history.is_empty() {
        tx.delete(BUCKET, user_id)
    } else {
        let stored = history.to_vec();
        tx.put(BUCKET, user_id, &stored)
    }
}

fn trim(history: &mut Vec<String>, limit: u32) {
    let limit = limit as usize;
    if history.len() > limit {
        history.drain(0..history.len() - limit);
    }
}

fn reused(plaintext: &str, history: &[String], current_hash: &str, limit: u32) -> bool {
    let mut seen = BTreeSet::new();
    if !current_hash.is_empty() {
        seen.insert(current_hash.to_owned());
        if crypto::password_matches(plaintext, current_hash) {
            return true;
        }
    }
    for hash in history.iter().rev().take(limit as usize) {
        if hash.is_empty() || !seen.insert(hash.clone()) {
            continue;
        }
        if crypto::password_matches(plaintext, hash) {
            return true;
        }
    }
    false
}

fn retain(history: &mut Vec<String>, current_hash: &str, hash: &str, limit: u32) {
    if !current_hash.is_empty()
        && current_hash != hash
        && !history.iter().any(|item| item == current_hash)
    {
        history.push(current_hash.to_owned());
    }
    if history.last().is_none_or(|item| item != hash) {
        history.retain(|item| item != hash);
        history.push(hash.to_owned());
    }
    trim(history, limit);
}

pub(crate) fn accept(
    tx: &Tx<'_>,
    limit: u32,
    user_id: &str,
    current_hash: &str,
    plaintext: &str,
    new_hash: &str,
) -> Result<()> {
    if new_hash.is_empty() || limit == 0 {
        return Ok(());
    }
    let mut history = load(tx, user_id)?;
    if reused(plaintext, &history, current_hash, limit) {
        return Err(Error::bad("Password was used recently"));
    }
    retain(&mut history, current_hash, new_hash, limit);
    save(tx, user_id, &history)
}

pub(crate) fn record_imported_hash(
    tx: &Tx<'_>,
    limit: u32,
    user_id: &str,
    current_hash: &str,
    hash: &str,
) -> Result<()> {
    if hash.is_empty() || limit == 0 {
        return Ok(());
    }
    let mut history = load(tx, user_id)?;
    retain(&mut history, current_hash, hash, limit);
    save(tx, user_id, &history)
}

pub(crate) fn note_rehash(
    tx: &Tx<'_>,
    limit: u32,
    user_id: &str,
    old_hash: &str,
    new_hash: &str,
) -> Result<()> {
    if new_hash.is_empty() || limit == 0 || old_hash == new_hash {
        return Ok(());
    }
    let mut history = load(tx, user_id)?;
    if let Some(index) = history.iter().position(|item| item == old_hash) {
        history[index] = new_hash.to_owned();
        let mut seen = BTreeSet::new();
        history.retain(|item| seen.insert(item.clone()));
        trim(&mut history, limit);
    } else {
        retain(&mut history, "", new_hash, limit);
    }
    save(tx, user_id, &history)
}
