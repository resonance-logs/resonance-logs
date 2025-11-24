use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs;

const DEFAULT_SKILL_JSON_RELATIVE: &str = "meter-data/SkillName.json";

/// Thread-safe cache for skill names, loaded lazily on first access
static SKILL_CACHE: Lazy<RwLock<HashMap<i32, String>>> =
    Lazy::new(|| RwLock::new(load_skill_names().unwrap_or_default()));

/// Loads skill names from the JSON file into a HashMap
fn load_skill_names() -> Result<HashMap<i32, String>, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(DEFAULT_SKILL_JSON_RELATIVE)?;
    let json: HashMap<String, String> = serde_json::from_str(&contents)?;

    let mut skill_map = HashMap::with_capacity(json.len());

    for (skill_id_str, skill_name) in json {
        if let Ok(skill_id) = skill_id_str.parse::<i32>() {
            skill_map.insert(skill_id, skill_name);
        }
    }

    Ok(skill_map)
}

/// Returns the best available name for the given skill id, reloading the cache if needed.
pub fn lookup(skill_id: i32) -> Option<String> {
    let cache = SKILL_CACHE.read();
    cache.get(&skill_id).cloned()
}

/// Manually reload the skill names cache from disk
pub fn reload_cache() -> Result<(), Box<dyn std::error::Error>> {
    let new_cache = load_skill_names()?;
    let mut cache = SKILL_CACHE.write();
    *cache = new_cache;
    Ok(())
}
