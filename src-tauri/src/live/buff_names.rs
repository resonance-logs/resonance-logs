use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const BUFF_JSON_RELATIVE: &str = "meter-data/BuffName.json";

#[derive(Debug, Deserialize)]
struct RawBuffEntry {
    #[serde(rename = "hasEnglishShortAndLong")]
    has_english_short_and_long: Option<bool>,
    #[serde(rename = "EnglishShort")]
    english_short: Option<String>,
    #[serde(rename = "EnglishLong")]
    english_long: Option<String>,
}

/// Cache stores tuples of (EnglishShort, EnglishLong) for entries that provide both.
static BUFF_CACHE: Lazy<RwLock<HashMap<i32, (String, String)>>> = Lazy::new(|| {
    let map = load_buff_names().unwrap_or_default();
    RwLock::new(map)
});

fn locate_buff_file() -> Option<PathBuf> {
    // Try relative path first
    let mut p = PathBuf::from(BUFF_JSON_RELATIVE);
    if p.exists() {
        return Some(p);
    }

    // Try src-tauri prefixed
    p = PathBuf::from(format!("src-tauri/{}", BUFF_JSON_RELATIVE));
    if p.exists() {
        return Some(p);
    }

    // Try exe dir
    if let Ok(mut exe_dir) = std::env::current_exe() {
        exe_dir.pop();
        let candidate = exe_dir.join(BUFF_JSON_RELATIVE);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn load_buff_names() -> Result<HashMap<i32, (String, String)>, Box<dyn std::error::Error>> {
    let path = match locate_buff_file() {
        Some(p) => p,
        None => return Ok(HashMap::new()),
    };

    let contents = fs::read_to_string(path)?;
    let v: serde_json::Value = serde_json::from_str(&contents)?;

    let mut buff_map: HashMap<i32, (String, String)> = HashMap::new();

    if let serde_json::Value::Object(map) = v {
        for (k, val) in map.into_iter() {
            if let Ok(buff_id) = k.parse::<i32>() {
                if let serde_json::Value::Object(obj) = val {
                    let raw: RawBuffEntry = match serde_json::from_value(serde_json::Value::Object(obj)) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };

                    if raw.has_english_short_and_long.unwrap_or(false) {
                        if let (Some(short), Some(long)) = (raw.english_short, raw.english_long) {
                            if !short.is_empty() && !long.is_empty() {
                                buff_map.insert(buff_id, (short, long));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(buff_map)
}

/// Returns the English short name for the given buff id if the entry declares
/// `hasEnglishShortAndLong` and provides both short and long names.
pub fn lookup(buff_id: i32) -> Option<String> {
    let cache = BUFF_CACHE.read();
    cache.get(&buff_id).map(|(s, _)| s.clone())
}

/// Returns both (EnglishShort, EnglishLong) when available.
pub fn lookup_full(buff_id: i32) -> Option<(String, String)> {
    let cache = BUFF_CACHE.read();
    cache.get(&buff_id).cloned()
}

/// Reload the cache from disk.
pub fn reload_cache() -> Result<(), Box<dyn std::error::Error>> {
    let new_map = load_buff_names()?;
    let mut cache = BUFF_CACHE.write();
    *cache = new_map;
    Ok(())
}
