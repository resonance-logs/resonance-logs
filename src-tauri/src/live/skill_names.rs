use log::warn;
use parking_lot::RwLock;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, SystemTime};

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

const DEFAULT_SKILL_JSON_RELATIVE: &str = "raw-game-files/4_Final/CombinedTranslatedWithManualOverrides.json";
const SKILL_JSON_ENV: &str = "RESONANCE_LOGS_SKILL_JSON";

pub fn lookup(skill_id: i32) -> Option<String> {
    let path = resolve_skill_json_path();
    ensure_cache(&path);

    let cache = SKILL_NAME_CACHE.read();
    cache.names.get(&skill_id).cloned()
}

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

fn string_from_value(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
}

fn resolve_skill_json_path() -> PathBuf {
    if let Ok(configured) = env::var(SKILL_JSON_ENV) {
        let path = PathBuf::from(configured);
        if path.is_absolute() {
            return path;
        }
        if let Ok(current_dir) = env::current_dir() {
            return current_dir.join(path);
        }
        return path;
    }

    let default_rel = Path::new(DEFAULT_SKILL_JSON_RELATIVE);

    if default_rel.is_absolute() {
        return default_rel.to_path_buf();
    }

    if let Ok(current_dir) = env::current_dir() {
        let candidate = current_dir.join(default_rel);
        if candidate.exists() {
            return candidate;
        }
    }

    if let Ok(mut exe_dir) = env::current_exe() {
        exe_dir.pop();
        let candidate = exe_dir.join(default_rel);
        if candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from(default_rel)
}

fn is_newer_than(current: SystemTime, previous: SystemTime) -> bool {
    match current.duration_since(previous) {
        Ok(duration) => duration > Duration::from_secs(0),
        Err(_) => true,
    }
}

fn log_once(cache: &mut SkillNameCache, message: String) {
    if cache.last_error.as_ref() != Some(&message) {
        warn!("{}", message);
        cache.last_error = Some(message);
    }
}
