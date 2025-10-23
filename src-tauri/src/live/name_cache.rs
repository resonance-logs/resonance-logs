use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::fs;

/// Default capacity for recent names cache.
pub const DEFAULT_RECENT_NAMES_CAPACITY: usize = 100;

/// RecentNamesCache stores the most-recently-seen player names keyed by UID.
/// It keeps insertion order in a small VecDeque and evicts oldest entries when
/// capacity is exceeded. This implementation is O(n)
#[derive(Debug, Clone)]
pub struct RecentNamesCache {
    capacity: usize,
    order: VecDeque<i64>, // front = oldest, back = newest
    map: HashMap<i64, String>,
}

impl RecentNamesCache {
    /// Create a cache with a specific capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            order: VecDeque::new(),
            map: HashMap::new(),
        }
    }

    /// Try to load a configured capacity from `capabilities/live.json`.
    /// If anything fails, fall back to DEFAULT_RECENT_NAMES_CAPACITY.
    pub fn capacity_from_capabilities() -> usize {
        // Attempt to read the file at `capabilities/live.json` relative to cwd.
        let path = "capabilities/live.json";
        match fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str::<Value>(&contents) {
                Ok(v) => match v.get("recent_names_capacity") {
                    Some(val) => val.as_u64().map(|u| u as usize).unwrap_or(DEFAULT_RECENT_NAMES_CAPACITY),
                    None => DEFAULT_RECENT_NAMES_CAPACITY,
                },
                Err(_) => DEFAULT_RECENT_NAMES_CAPACITY,
            },
            Err(_) => DEFAULT_RECENT_NAMES_CAPACITY,
        }
    }

    /// Add or update a name for uid. The uid becomes the most-recent entry.
    pub fn add_name(&mut self, uid: i64, name: String) {
        // If already present, update and move to back
        if self.map.contains_key(&uid) {
            self.map.insert(uid, name);
            // Remove existing occurrence from order
            self.order.retain(|&x| x != uid);
            self.order.push_back(uid);
            return;
        }

        // Insert new
        self.order.push_back(uid);
        self.map.insert(uid, name);

        // Evict oldest while exceeding capacity
        while self.order.len() > self.capacity {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            }
        }
    }

    /// Return the recent list as (uid, name) most-recent-first
    pub fn get_recent(&self) -> Vec<(i64, String)> {
        let mut out = Vec::with_capacity(self.order.len());
        for &uid in self.order.iter().rev() {
            if let Some(name) = self.map.get(&uid) {
                out.push((uid, name.clone()));
            }
        }
        out
    }

    /// Return a clone of the name for uid if present
    pub fn contains(&self, uid: i64) -> Option<String> {
        self.map.get(&uid).cloned()
    }

    /// Current size
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_add_and_get() {
        let mut c = RecentNamesCache::with_capacity(3);
        c.add_name(1, "Alice".to_string());
        c.add_name(2, "Bob".to_string());
        c.add_name(3, "Carol".to_string());
        let recent = c.get_recent();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].1, "Carol");
        assert_eq!(recent[1].1, "Bob");
        assert_eq!(recent[2].1, "Alice");
    }

    #[test]
    fn eviction_and_update() {
        let mut c = RecentNamesCache::with_capacity(2);
        c.add_name(1, "A".to_string());
        c.add_name(2, "B".to_string());
        c.add_name(3, "C".to_string());
        assert_eq!(c.len(), 2);
        assert!(c.contains(1).is_none());

        // Update existing uid moves it to most recent
        c.add_name(2, "B2".to_string());
        let recent = c.get_recent();
        assert_eq!(recent[0].0, 2);
        assert_eq!(recent[0].1, "B2");
    }
}
