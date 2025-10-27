use crate::live::opcodes_models::class::ClassSpec;
use blueprotobuf_lib::blueprotobuf::{EEntityType, SyncContainerData};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use tokio::sync::RwLock;
use windivert::layer::NetworkLayer;
use windivert::WinDivert;

#[derive(Debug, Default, Clone)]
pub struct Encounter {
    pub is_encounter_paused: bool,
    pub time_last_combat_packet_ms: u128, // in ms
    pub time_fight_start_ms: u128,        // in ms
    pub total_dmg: u128,
    pub total_heal: u128,
    pub local_player_uid: i64,
    pub entity_uid_to_entity: HashMap<i64, Entity>, // key: entity uid
    pub local_player: SyncContainerData,
}

// Use an async-aware RwLock so readers don't block the tokio runtime threads.
pub type EncounterMutex = RwLock<Encounter>;

#[derive(Debug, Default, Clone)]
pub struct Entity {
    pub name: String,
    pub entity_type: EEntityType,
    pub class_id: i32,
    pub class_spec: ClassSpec,
    pub ability_score: i32,
    pub level: i32,
    // Raw monster name captured from packet ATTR_NAME when available (monsters only)
    pub monster_name_packet: Option<String>,
    // Damage
    pub total_dmg: u128,
    pub crit_total_dmg: u128,
    pub crit_hits_dmg: u128,
    pub lucky_total_dmg: u128,
    pub lucky_hits_dmg: u128,
    pub hits_dmg: u128,
    pub skill_uid_to_dmg_skill: HashMap<i32, Skill>,
    // Healing
    pub total_heal: u128,
    pub crit_total_heal: u128,
    pub crit_hits_heal: u128,
    pub lucky_total_heal: u128,
    pub lucky_hits_heal: u128,
    pub hits_heal: u128,
    pub skill_uid_to_heal_skill: HashMap<i32, Skill>,
    // Tanked/Taken (damage received)
    pub total_taken: u128,
    pub crit_total_taken: u128,
    pub crit_hits_taken: u128,
    pub lucky_total_taken: u128,
    pub lucky_hits_taken: u128,
    pub hits_taken: u128,
    pub skill_uid_to_taken_skill: HashMap<i32, Skill>,

    // Monster metadata and per-target aggregates (for boss-only filtering)
    pub monster_type_id: Option<i32>,
    pub dmg_to_target: HashMap<i64, u128>,
    pub skill_dmg_to_target: HashMap<i32, HashMap<i64, u128>>,
}

#[derive(Debug, Default, Clone)]
pub struct Skill {
    pub total_value: u128,
    pub crit_total_value: u128,
    pub crit_hits: u128,
    pub lucky_total_value: u128,
    pub lucky_hits: u128,
    pub hits: u128,
}

static SKILL_NAMES: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let data = include_str!("../../meter-data/SkillName.json");
    serde_json::from_str(data).expect("invalid skills.json")
});

// Monster names mapping (id -> name)
static MONSTER_NAMES: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let data = include_str!("../../meter-data/MonsterName.json");
    serde_json::from_str(data).expect("invalid MonsterName.json")
});

/// Build a normalized set of boss names from MONSTER_NAMES.
///
/// Normalization rules:
/// - case-insensitive
/// - if the name starts with a boss prefix (e.g. "Boss", "Boss:", "Boss -"), strip the prefix and any following separators
/// - trim whitespace
///
/// Example: "Boss - Tempest Ogre" -> "tempest ogre" (stored in the set)
static BOSS_NORMALIZED_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    fn normalize_boss_name(s: &str) -> String {
        let mut s = s.trim().to_lowercase();
        if let Some(rest) = s.strip_prefix("boss") {
            // Strip typical separators after the boss label
            let rest = rest.trim_start_matches(|c: char| c == ' ' || c == '-' || c == ':' );
            return rest.trim().to_string();
        }
        s
    }

    let mut set = HashSet::new();
    for name in MONSTER_NAMES.values() {
        if name.to_lowercase().contains("boss") {
            let n = normalize_boss_name(name);
            if !n.is_empty() {
                set.insert(n);
            }
        }
    }
    set
});

impl Skill {
    pub fn get_skill_name(skill_uid: i32) -> String {
        SKILL_NAMES.get(&skill_uid.to_string()).map_or_else(
            || format!("UNKNOWN UNKNOWN ({skill_uid})"),
            |s| format!("{s} ({skill_uid})"),
        )
    }
}

impl Encounter {
    /// Reset only combat-specific state while preserving player identity fields and cache.
    ///
    /// Preserves:
    /// - is_encounter_paused
    /// - local_player_uid
    /// - local_player (sync container data)
    /// - entity_uid_to_entity identity fields (name, class, spec, ability score, level, type)
    ///
    /// Clears:
    /// - encounter totals and timestamps
    /// - per-entity combat counters and per-encounter skill maps
    pub fn reset_combat_state(&mut self) {
        // Reset encounter-level combat state
        self.time_last_combat_packet_ms = 0;
        self.time_fight_start_ms = 0;
        self.total_dmg = 0;
        self.total_heal = 0;

        // Reset per-entity combat stats while preserving identity
        for entity in self.entity_uid_to_entity.values_mut() {
            // Damage
            entity.total_dmg = 0;
            entity.crit_total_dmg = 0;
            entity.crit_hits_dmg = 0;
            entity.lucky_total_dmg = 0;
            entity.lucky_hits_dmg = 0;
            entity.hits_dmg = 0;
            entity.skill_uid_to_dmg_skill.clear();
            entity.dmg_to_target.clear();
            entity.skill_dmg_to_target.clear();

            // Healing
            entity.total_heal = 0;
            entity.crit_total_heal = 0;
            entity.crit_hits_heal = 0;
            entity.lucky_total_heal = 0;
            entity.lucky_hits_heal = 0;
            entity.hits_heal = 0;
            entity.skill_uid_to_heal_skill.clear();

            // Taken
            entity.total_taken = 0;
            entity.crit_total_taken = 0;
            entity.crit_hits_taken = 0;
            entity.lucky_total_taken = 0;
            entity.lucky_hits_taken = 0;
            entity.hits_taken = 0;
            entity.skill_uid_to_taken_skill.clear();
        }
    }
}

pub mod attr_type {
    pub const ATTR_NAME: i32 = 0x01;
    pub const ATTR_ID: i32 = 0x0a;
    pub const ATTR_PROFESSION_ID: i32 = 0xdc;
    pub const ATTR_FIGHT_POINT: i32 = 0x272e;
    pub const ATTR_LEVEL: i32 = 0x2710;
    // pub const ATTR_RANK_LEVEL: i32 = 0x274c;
    // pub const ATTR_CRI: i32 = 0x2b66;
    // pub const ATTR_LUCKY: i32 = 0x2b7a;
    // pub const ATTR_HP: i32 = 0x2c2e;
    // pub const ATTR_MAX_HP: i32 = 0x2c38;
    // pub const ATTR_ELEMENT_FLAG: i32 = 0x646d6c;
    // pub const ATTR_REDUCTION_LEVEL: i32 = 0x64696d;
    // pub const ATTR_REDUCTION_ID: i32 = 0x6f6c65;
    // pub const ATTR_ENERGY_FLAG: i32 = 0x543cd3c6;
}

// TODO: this logic needs to be severely cleaned up
pub mod class {
    pub const UNKNOWN: i32 = 0;
    pub const STORMBLADE: i32 = 1;
    pub const FROST_MAGE: i32 = 2;
    pub const WIND_KNIGHT: i32 = 4;
    pub const VERDANT_ORACLE: i32 = 5;
    pub const HEAVY_GUARDIAN: i32 = 9;
    pub const MARKSMAN: i32 = 11;
    pub const SHIELD_KNIGHT: i32 = 12;
    pub const BEAT_PERFORMER: i32 = 13;

    pub fn get_class_name(id: i32) -> String {
        String::from(match id {
            STORMBLADE => "Stormblade",
            FROST_MAGE => "Frost Mage",
            WIND_KNIGHT => "Wind Knight",
            VERDANT_ORACLE => "Verdant Oracle",
            HEAVY_GUARDIAN => "Heavy Guardian",
            MARKSMAN => "Marksman",
            SHIELD_KNIGHT => "Shield Knight",
            BEAT_PERFORMER => "Beat Performer",
            _ => "", // empty string for unknown
        })
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub enum ClassSpec {
        #[default]
        Unknown,
        // Stormblade
        Iaido,
        Moonstrike,
        // Frost Mage
        Icicle,
        Frostbeam,
        // Wind Knight
        Vanguard,
        Skyward,
        // Verdant Oracle
        Smite,
        Lifebind,
        // Heavy Guardian
        Earthfort,
        Block,
        // Marksman
        Wildpack,
        Falconry,
        // Shield Knight
        Recovery,
        Shield,
        // Beat Performer
        Dissonance,
        Concerto,
    }

    pub fn get_class_spec_from_skill_id(skill_id: i32) -> ClassSpec {
        match skill_id {
            1714 | 1734 => ClassSpec::Iaido,
            44701 | 179906 => ClassSpec::Moonstrike,

            120901 | 120902 => ClassSpec::Icicle,
            1241 => ClassSpec::Frostbeam,

            1405 | 1418 => ClassSpec::Vanguard,
            1419 => ClassSpec::Skyward,

            1518 | 1541 | 21402 => ClassSpec::Smite,
            20301 => ClassSpec::Lifebind,

            199902 => ClassSpec::Earthfort,
            1930 | 1931 | 1934 | 1935 => ClassSpec::Block,

            220112 | 2203622 => ClassSpec::Falconry,
            2292 | 1700820 | 1700825 | 1700827 => ClassSpec::Wildpack,

            2406 => ClassSpec::Recovery,
            2405 => ClassSpec::Shield,

            2306 => ClassSpec::Dissonance,
            2307 | 2361 | 55302 => ClassSpec::Concerto,
            _ => ClassSpec::Unknown,
        }
    }

    pub fn get_class_id_from_spec(class_spec: ClassSpec) -> i32 {
        match class_spec {
            ClassSpec::Iaido | ClassSpec::Moonstrike => STORMBLADE,
            ClassSpec::Icicle | ClassSpec::Frostbeam => FROST_MAGE,
            ClassSpec::Vanguard | ClassSpec::Skyward => WIND_KNIGHT,
            ClassSpec::Smite | ClassSpec::Lifebind => VERDANT_ORACLE,
            ClassSpec::Earthfort | ClassSpec::Block => HEAVY_GUARDIAN,
            ClassSpec::Wildpack | ClassSpec::Falconry => MARKSMAN,
            ClassSpec::Recovery | ClassSpec::Shield => SHIELD_KNIGHT,
            ClassSpec::Dissonance | ClassSpec::Concerto => BEAT_PERFORMER,
            ClassSpec::Unknown => UNKNOWN,
        }
    }

    pub fn get_class_spec(class_spec: ClassSpec) -> String {
        String::from(match class_spec {
            ClassSpec::Unknown => "",
            ClassSpec::Iaido => "Iaido",
            ClassSpec::Moonstrike => "Moonstrike",
            ClassSpec::Icicle => "Icicle",
            ClassSpec::Frostbeam => "Frostbeam",
            ClassSpec::Vanguard => "Vanguard",
            ClassSpec::Skyward => "Skyward",
            ClassSpec::Smite => "Smite",
            ClassSpec::Lifebind => "Lifebind",
            ClassSpec::Earthfort => "Earthfort",
            ClassSpec::Block => "Block",
            ClassSpec::Wildpack => "Wildpack",
            ClassSpec::Falconry => "Falconry",
            ClassSpec::Recovery => "Recovery",
            ClassSpec::Shield => "Shield",
            ClassSpec::Dissonance => "Dissonance",
            ClassSpec::Concerto => "Concerto",
        })
    }
}

impl Entity {
    /// Assign monster type id and update display name from mapping if available.
    pub fn set_monster_type(&mut self, monster_id: i32) {
        self.monster_type_id = Some(monster_id);
        if let Some(name) = MONSTER_NAMES.get(&monster_id.to_string()) {
            self.name = name.clone();
        }
    }

    /// Determine whether this entity is a boss based on its mapped or direct name.
    pub fn is_boss(&self) -> bool {
        if self.entity_type != EEntityType::EntMonster {
            return false;
        }
        // Helper to normalize names similar to how the boss set was built
        fn normalize_candidate_name(s: &str) -> String {
            let mut s = s.trim().to_lowercase();
            if let Some(rest) = s.strip_prefix("boss") {
                let rest = rest.trim_start_matches(|c: char| c == ' ' || c == '-' || c == ':' );
                return rest.trim().to_string();
            }
            s
        }

        // Prefer mapped monster name when id is known
        let candidate = if let Some(monster_id) = self.monster_type_id {
            MONSTER_NAMES
                .get(&monster_id.to_string())
                .cloned()
                .unwrap_or_else(|| self.name.clone())
        } else {
            self.name.clone()
        };

        if candidate.is_empty() {
            return false;
        }

        // Fast path: if the mapped name explicitly contains "Boss" (any case)
        if candidate.to_lowercase().contains("boss") {
            return true;
        }

        // Normalized match against known boss names (e.g. "Tempest Ogre" should match "Boss - Tempest Ogre")
        let normalized = normalize_candidate_name(&candidate);
        BOSS_NORMALIZED_NAMES.contains(&normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tempest_ogre_detected_as_boss_via_normalization() {
        // 10010 -> "Tempest Ogre" (no Boss prefix)
        let mut e = Entity::default();
        e.entity_type = EEntityType::EntMonster;
        e.set_monster_type(10010);
        assert_eq!(e.name, "Tempest Ogre");
        assert!(e.is_boss(), "Tempest Ogre should be recognized as a boss");
    }

    #[test]
    fn explicit_boss_prefix_is_boss() {
        // Use an id that maps to "Boss - Tempest Ogre"
        let mut e = Entity::default();
        e.entity_type = EEntityType::EntMonster;
        e.set_monster_type(20088); // Boss - Tempest Ogre
        assert!(e.name.to_lowercase().contains("boss"));
        assert!(e.is_boss());
    }

    #[test]
    fn non_boss_is_not_boss() {
        // Goblin Mage id mapping without boss label
        let mut e = Entity::default();
        e.entity_type = EEntityType::EntMonster;
        e.set_monster_type(40015); // Goblin Mage
        assert!(!e.is_boss());
    }
}
