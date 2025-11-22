use once_cell::sync::Lazy;
/// Module type and attribute mappings synchronized with C# implementation
/// from StarResonanceDps/ModuleType.cs and resonance-website backend
use std::collections::HashMap;

// Module Type IDs
pub const BASIC_ATTACK: i32 = 5500101;
pub const HIGH_PERFORMANCE_ATTACK: i32 = 5500102;
pub const EXCELLENT_ATTACK: i32 = 5500103;
pub const BASIC_HEALING: i32 = 5500201;
pub const HIGH_PERFORMANCE_HEALING: i32 = 5500202;
pub const EXCELLENT_HEALING: i32 = 5500203;
pub const BASIC_PROTECTION: i32 = 5500301;
pub const HIGH_PERFORMANCE_PROTECTION: i32 = 5500302;
pub const EXCELLENT_PROTECTION: i32 = 5500303;

// Basic Attribute IDs
pub const STRENGTH_BOOST: i32 = 1110;
pub const AGILITY_BOOST: i32 = 1111;
pub const INTELLIGENCE_BOOST: i32 = 1112;
pub const SPECIAL_ATTACK_DAMAGE: i32 = 1113;
pub const ELITE_STRIKE: i32 = 1114;
pub const SPECIAL_HEALING_BOOST: i32 = 1205;
pub const EXPERT_HEALING_BOOST: i32 = 1206;
pub const MAGIC_RESISTANCE: i32 = 1307;
pub const PHYSICAL_RESISTANCE: i32 = 1308;
pub const CASTING_FOCUS: i32 = 1407;
pub const ATTACK_SPEED_FOCUS: i32 = 1408;
pub const CRITICAL_FOCUS: i32 = 1409;
pub const LUCK_FOCUS: i32 = 1410;

// Special (Extreme) Attribute IDs
pub const EXTREME_DAMAGE_STACK: i32 = 2104;
pub const EXTREME_FLEXIBLE_MOVEMENT: i32 = 2105;
pub const EXTREME_LIFE_CONVERGENCE: i32 = 2204;
pub const EXTREME_EMERGENCY_MEASURES: i32 = 2205;
pub const EXTREME_DESPERATE_GUARDIAN: i32 = 2304;
pub const EXTREME_LIFE_FLUCTUATION: i32 = 2404;
pub const EXTREME_LIFE_DRAIN: i32 = 2405;
pub const EXTREME_TEAM_CRIT: i32 = 2406;

/// Module names mapping (config_id → name)
pub static MODULE_NAMES: Lazy<HashMap<i32, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(BASIC_ATTACK, "基础攻击");
    m.insert(HIGH_PERFORMANCE_ATTACK, "高性能攻击");
    m.insert(EXCELLENT_ATTACK, "卓越攻击");
    m.insert(BASIC_HEALING, "基础治疗");
    m.insert(HIGH_PERFORMANCE_HEALING, "高性能治疗");
    m.insert(EXCELLENT_HEALING, "卓越辅助");
    m.insert(BASIC_PROTECTION, "基础防护");
    m.insert(HIGH_PERFORMANCE_PROTECTION, "高性能守护");
    m.insert(EXCELLENT_PROTECTION, "卓越守护");
    m
});

/// Module attribute names mapping (part_id → name)
pub static MODULE_ATTR_NAMES: Lazy<HashMap<i32, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Basic attributes
    m.insert(STRENGTH_BOOST, "力量加持");
    m.insert(AGILITY_BOOST, "敏捷加持");
    m.insert(INTELLIGENCE_BOOST, "智力加持");
    m.insert(SPECIAL_ATTACK_DAMAGE, "特攻伤害");
    m.insert(ELITE_STRIKE, "精英打击");
    m.insert(SPECIAL_HEALING_BOOST, "特攻治疗加持");
    m.insert(EXPERT_HEALING_BOOST, "专精治疗加持");
    m.insert(MAGIC_RESISTANCE, "抵御魔法");
    m.insert(PHYSICAL_RESISTANCE, "抵御物理");
    m.insert(CASTING_FOCUS, "施法专注");
    m.insert(ATTACK_SPEED_FOCUS, "攻速专注");
    m.insert(CRITICAL_FOCUS, "暴击专注");
    m.insert(LUCK_FOCUS, "幸运专注");

    // Special (Extreme) attributes
    m.insert(EXTREME_DAMAGE_STACK, "极-伤害叠加");
    m.insert(EXTREME_FLEXIBLE_MOVEMENT, "极-灵活身法");
    m.insert(EXTREME_LIFE_CONVERGENCE, "极-生命凝聚");
    m.insert(EXTREME_EMERGENCY_MEASURES, "极-急救措施");
    m.insert(EXTREME_DESPERATE_GUARDIAN, "极-绝境守护");
    m.insert(EXTREME_LIFE_FLUCTUATION, "极-生命波动");
    m.insert(EXTREME_LIFE_DRAIN, "极-生命汲取");
    m.insert(EXTREME_TEAM_CRIT, "极-全队幸暴");
    m
});

/// Module category mapping (config_id → category)
pub static MODULE_CATEGORY_MAP: Lazy<HashMap<i32, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(BASIC_ATTACK, "ATTACK");
    m.insert(HIGH_PERFORMANCE_ATTACK, "ATTACK");
    m.insert(EXCELLENT_ATTACK, "ATTACK");
    m.insert(BASIC_PROTECTION, "DEFENSE");
    m.insert(HIGH_PERFORMANCE_PROTECTION, "DEFENSE");
    m.insert(EXCELLENT_PROTECTION, "DEFENSE");
    m.insert(BASIC_HEALING, "SUPPORT");
    m.insert(HIGH_PERFORMANCE_HEALING, "SUPPORT");
    m.insert(EXCELLENT_HEALING, "SUPPORT");
    m
});

/// Basic attribute IDs set (for type classification)
pub static BASIC_ATTR_IDS: Lazy<std::collections::HashSet<i32>> = Lazy::new(|| {
    vec![
        STRENGTH_BOOST,
        AGILITY_BOOST,
        INTELLIGENCE_BOOST,
        SPECIAL_ATTACK_DAMAGE,
        ELITE_STRIKE,
        SPECIAL_HEALING_BOOST,
        EXPERT_HEALING_BOOST,
        MAGIC_RESISTANCE,
        PHYSICAL_RESISTANCE,
        CASTING_FOCUS,
        ATTACK_SPEED_FOCUS,
        CRITICAL_FOCUS,
        LUCK_FOCUS,
    ]
    .into_iter()
    .collect()
});

/// Special attribute IDs set (for type classification)
pub static SPECIAL_ATTR_IDS: Lazy<std::collections::HashSet<i32>> = Lazy::new(|| {
    vec![
        EXTREME_DAMAGE_STACK,
        EXTREME_FLEXIBLE_MOVEMENT,
        EXTREME_LIFE_CONVERGENCE,
        EXTREME_EMERGENCY_MEASURES,
        EXTREME_DESPERATE_GUARDIAN,
        EXTREME_LIFE_FLUCTUATION,
        EXTREME_LIFE_DRAIN,
        EXTREME_TEAM_CRIT,
    ]
    .into_iter()
    .collect()
});

/// Get attribute type ("basic" or "special") by part ID
pub fn get_attr_type(part_id: i32) -> &'static str {
    if BASIC_ATTR_IDS.contains(&part_id) {
        "basic"
    } else if SPECIAL_ATTR_IDS.contains(&part_id) {
        "special"
    } else {
        "basic" // Default to basic for unknown IDs
    }
}

/// Get module name by config ID
pub fn get_module_name(config_id: i32) -> Option<&'static str> {
    MODULE_NAMES.get(&config_id).copied()
}

/// Get module category by config ID
pub fn get_module_category(config_id: i32) -> Option<&'static str> {
    MODULE_CATEGORY_MAP.get(&config_id).copied()
}

/// Get attribute name by part ID
pub fn get_attr_name(part_id: i32) -> Option<&'static str> {
    MODULE_ATTR_NAMES.get(&part_id).copied()
}
