//! Per-node request counters: one fixed 60 s window per (grouped address, category).
use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    time::{Duration, Instant},
};

const WINDOW: Duration = Duration::from_secs(60);
type Key = (IpAddr, &'static str);

/// Windows are kept in the order they started, so expiring and evicting them is amortized
/// O(1). A full table evicts its oldest window instead of refusing clients it has not seen:
/// filling it with many addresses resets other clients' counts early, but never locks
/// them out.
#[doc(hidden)]
pub struct RateTable {
    windows: HashMap<Key, (Instant, u32)>,
    /// One entry per window in `windows`, oldest first.
    order: VecDeque<(Instant, Key)>,
    capacity: usize,
}
impl RateTable {
    pub fn new(capacity: usize) -> Self {
        Self {
            windows: HashMap::new(),
            order: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }
    /// Counts one request at `at`, which never goes backwards between calls. True when
    /// `key` is over `limit` in its current window.
    pub fn hit(&mut self, key: Key, limit: u32, at: Instant) -> bool {
        while self
            .order
            .front()
            .is_some_and(|(start, _)| at.saturating_duration_since(*start) >= WINDOW)
        {
            self.evict_oldest();
        }
        if !self.windows.contains_key(&key) {
            if self.windows.len() >= self.capacity {
                self.evict_oldest();
            }
            self.windows.insert(key, (at, 0));
            self.order.push_back((at, key));
        }
        let (_, count) = self.windows.get_mut(&key).expect("inserted above");
        *count = count.saturating_add(1);
        *count > limit
    }
    pub fn len(&self) -> usize {
        self.windows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }
    fn evict_oldest(&mut self) {
        if let Some((_, key)) = self.order.pop_front() {
            self.windows.remove(&key);
        }
    }
}
