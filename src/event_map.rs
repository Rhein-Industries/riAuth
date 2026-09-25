//! Read-only aggregate of coordinates operators already stored.
//!
//! A point comes from `details.location` on the audit event or, when that
//! field is absent, from the actor's user attribute `location`. IP addresses
//! are never consulted, and this view does not write audit records.

use crate::{
    core::Core,
    error::{Error, Result},
    model::{Audit, User},
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

/// Events read from the audit log, newest first, before the response stops.
pub const MAX_SCAN: usize = 10_000;
/// Rounded cells returned. Further cells are counted in `omitted_cells`.
pub const MAX_POINTS: usize = 500;
const PAGE: usize = 256;

#[derive(Debug, Clone)]
pub struct MapQuery {
    pub since: Option<u64>,
    pub until: Option<u64>,
    pub action_prefix: Option<String>,
}

impl MapQuery {
    fn normalize(self) -> Result<Self> {
        let MapQuery {
            since,
            until,
            action_prefix,
        } = self;
        let action_prefix = match action_prefix {
            Some(value) => {
                if value.len() > 128 || value.chars().any(char::is_control) {
                    return Err(Error::bad(
                        "Action filter must be at most 128 characters without control characters",
                    ));
                }
                let value = value.trim();
                if value.is_empty() {
                    None
                } else {
                    Some(value.to_owned())
                }
            }
            None => None,
        };
        if let (Some(since), Some(until)) = (since, until)
            && since > until
        {
            return Err(Error::bad("The map time range ends before it starts"));
        }
        Ok(Self {
            since,
            until,
            action_prefix,
        })
    }
}

#[derive(Clone)]
struct Location {
    latitude: f64,
    longitude: f64,
    label: Option<String>,
}

#[derive(Default)]
struct Cell {
    count: u64,
    label: Option<String>,
    conflict: bool,
}

impl Core {
    pub fn audit_map(&self, token: &str, query: MapQuery) -> Result<Value> {
        self.store.read(|tx| {
            self.management(tx, token, "audit.read", "audit/events")?;
            aggregate(tx, &query.normalize()?)
        })
    }

    /// Browser administrators already hold every human management permission.
    /// Agents have no portal cookie; they use [`Self::audit_map`].
    pub fn audit_map_browser(&self, cookie: Option<&str>, query: MapQuery) -> Result<Value> {
        self.store.read(|tx| {
            let session = self
                .browser_session(tx, cookie)?
                .ok_or_else(Error::unauthorized)?;
            let user = self.identity_user(tx, &session.identity)?;
            if !user.admin {
                return Err(Error::forbidden());
            }
            aggregate(tx, &query.normalize()?)
        })
    }
}

fn aggregate(tx: &Tx<'_>, query: &MapQuery) -> Result<Value> {
    let mut cursor = None;
    let mut scanned = 0usize;
    let mut matched = 0u64;
    let mut unknown = 0u64;
    let mut window_done = false;
    let mut cells = BTreeMap::<(i32, i32), Cell>::new();
    let mut users = HashMap::<String, Option<Location>>::new();
    while scanned < MAX_SCAN && !window_done {
        let room = MAX_SCAN - scanned;
        let batch = tx.scan_reverse::<Audit>("audit", cursor.as_deref(), PAGE.min(room))?;
        if batch.is_empty() {
            window_done = true;
            break;
        }
        for (key, event) in batch {
            cursor = Some(key);
            scanned += 1;
            let older = query.since.is_some_and(|since| event.at < since);
            let newer = query.until.is_some_and(|until| event.at > until);
            if older {
                window_done = true;
            } else if !newer
                && query
                    .action_prefix
                    .as_ref()
                    .is_none_or(|prefix| event.action.starts_with(prefix))
            {
                matched = matched.saturating_add(1);
                if let Some(location) = location_for(tx, &event, &mut users)? {
                    add_cell(&mut cells, &location);
                } else {
                    unknown = unknown.saturating_add(1);
                }
            }
            if window_done || scanned >= MAX_SCAN {
                break;
            }
        }
    }
    let truncated = if window_done {
        false
    } else {
        match tx
            .scan_reverse::<Audit>("audit", cursor.as_deref(), 1)?
            .first()
        {
            None => false,
            Some((_, event)) => query.since.is_none_or(|since| event.at >= since),
        }
    };
    let mut ranked: Vec<_> = cells.into_iter().collect();
    ranked.sort_by(|left, right| {
        right
            .1
            .count
            .cmp(&left.1.count)
            .then(left.0.0.cmp(&right.0.0))
            .then(left.0.1.cmp(&right.0.1))
    });
    let omitted_cells = ranked.len().saturating_sub(MAX_POINTS) as u64;
    ranked.truncate(MAX_POINTS);
    let mut points = Vec::with_capacity(ranked.len());
    for ((latitude, longitude), cell) in ranked {
        let mut point = json!({
            "latitude": decimal(latitude)?,
            "longitude": decimal(longitude)?,
            "count": cell.count,
        });
        if let Some(label) = cell.label {
            point["label"] = Value::String(label);
        }
        points.push(point);
    }
    Ok(json!({
        "points": points,
        "unknown": unknown,
        "scanned": u64::try_from(scanned).map_err(Error::internal)?,
        "matched": matched,
        "truncated": truncated,
        "omitted_cells": omitted_cells,
    }))
}

fn location_for(
    tx: &Tx<'_>,
    event: &Audit,
    cache: &mut HashMap<String, Option<Location>>,
) -> Result<Option<Location>> {
    // A present `location` key is authoritative, even when it cannot be plotted.
    if let Some(value) = event.details.get("location") {
        return Ok(parse_location(value));
    }
    if let Some(cached) = cache.get(&event.actor) {
        return Ok(cached.clone());
    }
    let location = tx
        .get::<User>("users", &event.actor)?
        .and_then(|user| user.attributes.get("location").and_then(parse_location));
    cache.insert(event.actor.clone(), location.clone());
    Ok(location)
}

fn parse_location(value: &Value) -> Option<Location> {
    let object = value.as_object()?;
    let latitude = object.get("latitude").and_then(Value::as_f64)?;
    let longitude = object.get("longitude").and_then(Value::as_f64)?;
    if !latitude.is_finite()
        || !longitude.is_finite()
        || !(-90.0..=90.0).contains(&latitude)
        || !(-180.0..=180.0).contains(&longitude)
    {
        return None;
    }
    Some(Location {
        latitude,
        longitude,
        label: object.get("label").and_then(parse_label),
    })
}

fn parse_label(value: &Value) -> Option<String> {
    let text = value.as_str()?.trim();
    if text.is_empty() || text.len() > 80 || text.chars().any(char::is_control) {
        None
    } else {
        Some(text.to_owned())
    }
}

fn add_cell(cells: &mut BTreeMap<(i32, i32), Cell>, location: &Location) {
    let cell = cells
        .entry(cell_key(location.latitude, location.longitude))
        .or_default();
    cell.count = cell.count.saturating_add(1);
    if cell.conflict {
        return;
    }
    match (&cell.label, &location.label) {
        (_, None) => {}
        (None, Some(label)) => cell.label = Some(label.clone()),
        (Some(existing), Some(label)) if existing == label => {}
        (Some(_), Some(_)) => {
            cell.label = None;
            cell.conflict = true;
        }
    }
}

/// Nearest 0.1 degree, halfway cases away from zero. ±180° share one cell.
fn cell_key(latitude: f64, longitude: f64) -> (i32, i32) {
    let latitude = round_tenths(latitude).clamp(-900, 900);
    let longitude = round_tenths(longitude);
    let longitude = if (-1800..1800).contains(&longitude) {
        longitude
    } else {
        -1800
    };
    (latitude, longitude)
}

fn round_tenths(value: f64) -> i32 {
    let scaled = (value * 10.0).round();
    if !scaled.is_finite() {
        return 0;
    }
    scaled.clamp(i32::MIN as f64, i32::MAX as f64) as i32
}

fn decimal(tenths: i32) -> Result<Value> {
    let negative = tenths < 0;
    let absolute = tenths.unsigned_abs();
    let text = format!(
        "{}{}.{}",
        if negative { "-" } else { "" },
        absolute / 10,
        absolute % 10
    );
    serde_json::from_str(&text).map_err(Error::internal)
}
