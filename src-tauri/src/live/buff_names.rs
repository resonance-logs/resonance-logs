use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const DEFAULT_BUFF_JSON_RELATIVE: &str = "meter-data/BuffName.json";

#[derive(Debug, Deserialize)]
struct BuffEntry {
    #[serde(rename = "BuffTable_Clean.json")]
    buff_table: Option<BuffTableEntry>,
    #[serde(rename = "EnglishShortManualOverride")]
    manual_override: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuffTableEntry {
    #[serde(rename = "ChineseShort")]
    chinese_short: Option<String>,
    #[serde(rename = "AIEnglishShort")]
    ai_english_short: Option<String>,
}

/// Thread-safe cache for buff names, loaded lazily on first access
static BUFF_CACHE: Lazy<RwLock<HashMap<i32, String>>> =
    Lazy::new(|| RwLock::new(load_buff_names().unwrap_or_default()));

/// Loads buff names from the JSON file into a HashMap
fn load_buff_names() -> Result<HashMap<i32, String>, Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(DEFAULT_BUFF_JSON_RELATIVE);

    // Fallback paths similar to skill_names/scene_names
    if !path.exists() {
        path = PathBuf::from(format!("src-tauri/{}", DEFAULT_BUFF_JSON_RELATIVE));
    }

    if !path.exists() {
        if let Ok(mut exe_dir) = std::env::current_exe() {
            exe_dir.pop();
            if exe_dir.join(DEFAULT_BUFF_JSON_RELATIVE).exists() {
                path = exe_dir.join(DEFAULT_BUFF_JSON_RELATIVE);
            }
        }
    }

    let contents = fs::read_to_string(path)?;
    let json: HashMap<String, BuffEntry> = serde_json::from_str(&contents)?;

    let mut buff_map = HashMap::with_capacity(json.len());

    for (buff_id_str, entry) in json {
        if let Ok(buff_id) = buff_id_str.parse::<i32>() {
            // Priority: Manual Override -> AI English -> Chinese -> Default
            let name = if let Some(manual) = entry.manual_override {
                manual
            } else if let Some(table) = entry.buff_table {
                if let Some(eng) = table.ai_english_short {
                    eng
                } else if let Some(cn) = table.chinese_short {
                    cn
                } else {
                    continue;
                }
            } else {
                continue;
            };

            if !name.is_empty() {
                buff_map.insert(buff_id, name);
            }
        }
    }

    Ok(buff_map)
}

/// Returns the best available name for the given buff id.
pub fn lookup(buff_id: i32) -> Option<String> {
    let cache = BUFF_CACHE.read();
    cache.get(&buff_id).cloned()
}

/// Manually reload the buff names cache from disk
pub fn reload_cache() -> Result<(), Box<dyn std::error::Error>> {
    let new_cache = load_buff_names()?;
    let mut cache = BUFF_CACHE.write();
    *cache = new_cache;
    Ok(())
}
