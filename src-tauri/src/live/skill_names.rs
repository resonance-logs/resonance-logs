use log::warn;
use parking_lot::RwLock;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, SystemTime};

/// Stores cached skill names and their metadata to minimize JSON reloads.
#[derive(Default)]
struct SkillNameCache {
    names: HashMap<i32, String>,
    last_modified: Option<SystemTime>,
    last_path: Option<PathBuf>,
    last_error: Option<String>,
}

static SKILL_NAME_CACHE: LazyLock<RwLock<SkillNameCache>> = LazyLock::new(|| {
    RwLock::new(SkillNameCache {
        names: HashMap::new(),
        last_modified: None,
        last_path: None,
        last_error: None,
    })
});

const DEFAULT_SKILL_JSON_RELATIVE: &str =
    "raw-game-files/4_Final/CombinedTranslatedWithManualOverrides.json";
const SKILL_JSON_ENV: &str = "RESONANCE_LOGS_SKILL_JSON";

/// Returns the best available name for the given skill id, reloading the cache if needed.
pub fn lookup(skill_id: i32) -> Option<String> {
    let path = resolve_skill_json_path();
    ensure_cache(&path);

    let cache = SKILL_NAME_CACHE.read();
    cache.names.get(&skill_id).cloned()
}

/// Ensures the in-memory cache is populated and refreshed when the JSON source changes.
fn ensure_cache(path: &Path) {
    let metadata = fs::metadata(path);
    let modified = metadata.as_ref().ok().and_then(|m| m.modified().ok());

    let mut cache = SKILL_NAME_CACHE.write();
    let path_changed = cache
        .last_path
        .as_ref()
        .map_or(true, |last_path| last_path != path);

    let should_reload = path_changed
        || cache.names.is_empty()
        || modified.map_or(false, |curr| {
            cache
                .last_modified
                .map_or(true, |prev| is_newer_than(curr, prev))
        });

    if should_reload {
        match load_skill_names(path) {
            Ok(names) => {
                cache.names = names;
                cache.last_modified = modified;
                cache.last_path = Some(path.to_path_buf());
                cache.last_error = None;
            }
            Err(err) => {
                log_once(&mut cache, err);
                cache.names.clear();
                cache.last_modified = modified;
                cache.last_path = Some(path.to_path_buf());
            }
        }
        return;
    }

    if let Err(err) = metadata {
        log_once(
            &mut cache,
            format!(
                "skill names json not accessible at {}: {}",
                path.display(),
                err
            ),
        );
    }
}

/// Loads the skill names JSON file and builds a lookup map from id to display name.
fn load_skill_names(path: &Path) -> Result<HashMap<i32, String>, String> {
    let data = fs::read_to_string(path).map_err(|err| {
        format!(
            "failed to read skill names json at {}: {}",
            path.display(),
            err
        )
    })?;

    let parsed: Value = serde_json::from_str(&data).map_err(|err| {
        format!(
            "failed to parse skill names json at {}: {}",
            path.display(),
            err
        )
    })?;

    let mut names = HashMap::new();
    if let Value::Object(root) = parsed {
        for (id_str, entry) in root {
            let entry_obj = match entry {
                Value::Object(obj) => obj,
                _ => continue,
            };

            let Ok(skill_id) = id_str.parse::<i32>() else {
                continue;
            };

            if let Some(name) = extract_name(&entry_obj) {
                names.insert(skill_id, name);
            }
        }
    }

    Ok(names)
}

/// Picks the preferred name from a skill entry using the configured priority sequence.
fn extract_name(entry: &Map<String, Value>) -> Option<String> {
    enum Selector {
        TopLevel(&'static str),
        Nested(&'static str, &'static str),
    }

    const PRIORITY: &[Selector] = &[
        Selector::TopLevel("EnglishShortManualOverride"),
        Selector::Nested("RecountTable_Clean.json", "EnglishShort"),
        Selector::Nested("SkillTable_Clean.json", "EnglishShort"),
        Selector::Nested("RecountTable_Clean.json", "AIEnglishShort"),
        Selector::Nested("SkillTable_Clean.json", "AIEnglishShort"),
        Selector::TopLevel("AIEnglishShort"),
        Selector::Nested("skill_names_Clean.json", "EnglishShort"),
        Selector::Nested("skill_names_Clean.json", "AIEnglishShort"),
        Selector::Nested("BuffTable_Clean.json", "EnglishShort"),
        Selector::Nested("BuffTable_Clean.json", "AIEnglishShort"),
        Selector::TopLevel("ChineseShort"),
        Selector::Nested("SkillTable_Clean.json", "ChineseShort"),
        Selector::Nested("RecountTable_Clean.json", "ChineseShort"),
        Selector::Nested("skill_names_Clean.json", "ChineseShort"),
        Selector::Nested("BuffTable_Clean.json", "ChineseShort"),
    ];

    for selector in PRIORITY {
        let candidate = match selector {
            Selector::TopLevel(key) => string_from_value(entry.get(*key)),
            Selector::Nested(parent, key) => entry
                .get(*parent)
                .and_then(Value::as_object)
                .and_then(|obj| string_from_value(obj.get(*key))),
        };

        if let Some(name) = candidate {
            return Some(name);
        }
    }

    None
}

/// Converts a JSON value to a trimmed string if it contains non-empty text.
fn string_from_value(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
}

/// Resolves the path to the skill names JSON, honoring environment overrides and sensible defaults.
fn resolve_skill_json_path() -> PathBuf {
    if let Ok(configured) = env::var(SKILL_JSON_ENV) {
        let path = PathBuf::from(configured);
        if path.is_absolute() {
            return path;
        }
        if let Some(found) = resolve_relative_path(path.as_path()) {
            return found;
        }
        return path;
    }

    let default_rel = Path::new(DEFAULT_SKILL_JSON_RELATIVE);

    if default_rel.is_absolute() {
        return default_rel.to_path_buf();
    }

    if let Some(found) = resolve_relative_path(default_rel) {
        return found;
    }

    PathBuf::from(default_rel)
}

fn resolve_relative_path(relative: &Path) -> Option<PathBuf> {
    if let Ok(current_dir) = env::current_dir() {
        if let Some(found) = search_upwards(&current_dir, relative) {
            return Some(found);
        }
    }

    if let Ok(mut exe_dir) = env::current_exe() {
        exe_dir.pop();
        if let Some(found) = search_upwards(&exe_dir, relative) {
            return Some(found);
        }
    }

    None
}

fn search_upwards(start: &Path, relative: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let candidate = current.join(relative);
        if candidate.exists() {
            return Some(candidate);
        }

        if !current.pop() {
            break;
        }
    }

    None
}

/// Returns true if the current timestamp represents a newer value than the previous one.
fn is_newer_than(current: SystemTime, previous: SystemTime) -> bool {
    match current.duration_since(previous) {
        Ok(duration) => duration > Duration::from_secs(0),
        Err(_) => true,
    }
}

/// Logs an error message only once to avoid spamming repeated failures.
fn log_once(cache: &mut SkillNameCache, message: String) {
    if cache.last_error.as_ref() != Some(&message) {
        warn!("{}", message);
        cache.last_error = Some(message);
    }
}
