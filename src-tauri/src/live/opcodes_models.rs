use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use blueprotobuf_lib::blueprotobuf::EEntityType;

#[derive(Debug, Default, Clone)]
pub struct Encounter {
    pub time_last_combat_packet: u128, // in ms todo:
    pub time_fight_start: u128, // in ms todo:
    pub local_player_uid: i64, // todo: get from SyncToMeDeltaInfo
    pub entity_uid_to_entity: HashMap<i64, Entity>, // k: entity uid
}

pub type EncounterMutex = Mutex<Encounter>;

#[derive(Debug, Default, Clone)]
pub struct Entity {
    pub name: String,
    pub entity_type: EEntityType,
    pub class_id: i32,
    pub ability_score: i32,
    pub level: i32,
    pub damage_stats: DamageStats,
    pub skill_uid_to_skill: HashMap<i32, Skill>,
}

#[derive(Debug, Default, Clone)]
pub struct DamageStats {
    pub damage_dealt: u128,
}

#[derive(Debug, Default, Clone)]
pub struct Skill {
    pub total_damage: u128,
    // pub casts: i64,
    // pub hits: i64,
    // pub crits: i64,
}

static SKILL_NAMES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let data = include_str!("../../meter-data/SkillName.json");
    serde_json::from_str(data).expect("invalid skills.json")
});

impl Skill {
    pub fn get_skill_name(skill_uid: i32) -> String {
        SKILL_NAMES
            .get(&skill_uid.to_string())
            .cloned()
            .unwrap_or_else(|| format!("Unknown({})", skill_uid))
    }
}



pub mod attr_type {
    pub const ATTR_NAME: i32 = 0x01;
    // pub const ATTR_ID: i32 = 0x0a;
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

pub mod class {
    pub const STORMBLADE: i32 = 1;
    pub const FROST_MAGE: i32 = 2;
    pub const WIND_KNIGHT: i32 = 4;
    pub const VERDANT_ORACLE: i32 = 5;
    pub const HEAVY_GUARDIAN: i32 = 9;
    pub const MARKSMAN: i32 = 11;
    pub const SHIELD_KNIGHT: i32 = 12;
    pub const SOUL_MUSICIAN: i32 = 13;

    pub fn to_string(id: i32) -> String {
        match id {
            STORMBLADE => String::from("Stormblade"),
            FROST_MAGE => String::from("Frost Mage"),
            WIND_KNIGHT => String::from("Wind Knight"),
            VERDANT_ORACLE => String::from("Verdant Oracle"),
            HEAVY_GUARDIAN => String::from("Heavy Guardian"),
            MARKSMAN => String::from("Marksman"),
            SHIELD_KNIGHT => String::from("Shield Knight"),
            SOUL_MUSICIAN => String::from("Soul Musician"),
            _ => String::new(), // empty string for unknown
        }
    }
}
