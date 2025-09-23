use crate::live::commands_models::{DPSRow, DPSRows, DPSSkillBreakdownWindow, DPSWindow, HeaderInfo, SkillRow};
use crate::live::opcodes_models::{EncounterMutex, Skill, class};
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::info;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ops::{Div, DivAssign};

#[tauri::command]
#[specta::specta]
pub fn test_get_damage_row(state: tauri::State<'_, EncounterMutex>) -> DPSWindow {
    DPSWindow {
        dps_rows: vec![
            DPSRow {
                uid: 10000001,
                name: "Stormblade".to_string(),
                class: "Stormblade".to_string(),
                ability_score: 1500,
                total_dmg: 100_000,
                dps: 10_000.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
            },
            DPSRow {
                uid: 10000002,
                name: "Frost Mage".to_string(),
                class: "Frost Mage".to_string(),
                ability_score: 1500,
                total_dmg: 90_000,
                dps: 6_000.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
            },
            DPSRow {
                uid: 10000003,
                name: "Wind Knight".to_string(),
                class: "Wind Knight".to_string(),
                ability_score: 1500,
                total_dmg: 80_000,
                dps: 6_000.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
            },
            DPSRow {
                uid: 10000004,
                name: "Verdant Oracle".to_string(),
                class: "Verdant Oracle".to_string(),
                ability_score: 1500,
                total_dmg: 70_000,
                dps: 6_000.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
            },
            DPSRow {
                uid: 10000005,
                name: "Heavy Guardian".to_string(),
                class: "Heavy Guardian".to_string(),
                ability_score: 1500,
                total_dmg: 60_000,
                dps: 6_000.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
            },
            DPSRow {
                uid: 10000006,
                name: "Marksman".to_string(),
                class: "Marksman".to_string(),
                ability_score: 1500,
                total_dmg: 60_000,
                dps: 6_000.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
            },
            DPSRow {
                uid: 10000007,
                name: "Shield Knight".to_string(),
                class: "Shield Knight".to_string(),
                ability_score: 1500,
                total_dmg: 50_000,
                dps: 6_000.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
            },
            DPSRow {
                uid: 10000008,
                name: "Beat Performer".to_string(),
                class: "Beat Performer".to_string(),
                ability_score: 1500,
                total_dmg: 10_000,
                dps: 6_000.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
            },
            DPSRow {
                uid: 10000009,
                name: "Blank Class".to_string(),
                class: "blank".to_string(),
                ability_score: 1500,
                total_dmg: 10_000,
                dps: 6_000.6,
                crit_rate: 0.25,
                crit_dmg_rate: 2.0,
                lucky_rate: 0.10,
                lucky_dmg_rate: 1.5,
                hits: 200,
                hits_per_second: 3.3,
            },
        ],
        total_dmg: 100_000,
        elapsed_ms: 10_000, // 10 seconds
    }
}

#[tauri::command]
#[specta::specta]
pub fn test_get_skill_row(state: tauri::State<'_, EncounterMutex>, player_uid: String) -> DPSSkillBreakdownWindow {
    DPSSkillBreakdownWindow {
        curr_player: DPSRow {
            uid: 10000001,
            name: "Stormblade".to_string(),
            class: "Stormblade".to_string(),
            ability_score: 1500,
            total_dmg: 100_000,
            dps: 10_000.6,
            crit_rate: 0.25,
            crit_dmg_rate: 2.0,
            lucky_rate: 0.10,
            lucky_dmg_rate: 1.5,
            hits: 200,
            hits_per_second: 3.3,
        },
        skill_rows: vec![
            SkillRow {
                name: "Skill 1".to_string(),
                total_dmg: 100_000,
                dps: 5_000.0,
                crit_rate: 0.30,
                crit_dmg_rate: 2.1,
                lucky_rate: 0.12,
                lucky_dmg_rate: 1.4,
                hits: 80,
                hits_per_second: 1.5,
            },
            SkillRow {
                name: "Skill 2".to_string(),
                total_dmg: 50_000,
                dps: 7_345.6,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120,
                hits_per_second: 1.8,
            },
            SkillRow {
                name: "Skill 3".to_string(),
                total_dmg: 33_000,
                dps: 7_345.6,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120,
                hits_per_second: 1.8,
            },
            SkillRow {
                name: "Skill 4".to_string(),
                total_dmg: 23_000,
                dps: 7_345.6,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120,
                hits_per_second: 1.8,
            },
            SkillRow {
                name: "Skill 5".to_string(),
                total_dmg: 11_000,
                dps: 7_345.6,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120,
                hits_per_second: 1.8,
            },
            SkillRow {
                name: "Skill 6".to_string(),
                total_dmg: 1_000,
                dps: 7_345.6,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120,
                hits_per_second: 1.8,
            },
            SkillRow {
                name: "Skill 7".to_string(),
                total_dmg: 400,
                dps: 7_345.6,
                crit_rate: 0.20,
                crit_dmg_rate: 1.9,
                lucky_rate: 0.08,
                lucky_dmg_rate: 1.3,
                hits: 120,
                hits_per_second: 1.8,
            },
        ],
        total_dmg: 100_000,
        elapsed_ms: 10_000, // 10 seconds
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_header_info(state: tauri::State<'_, EncounterMutex>) -> HeaderInfo {
    let encounter = state.lock().unwrap();

    let mut header_info = HeaderInfo::default();

    let time_elapsed_ms = encounter.time_last_combat_packet.saturating_sub(encounter.time_fight_start);
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;
    for (&_entity_uid, entity) in &encounter.entity_uid_to_entity {
        header_info.total_dmg += entity.total_dmg;
    }
    header_info.total_dps = header_info.total_dmg as f64 / time_elapsed_secs.max(1.0);
    header_info.elapsed_ms = time_elapsed_ms;

    header_info
}

#[tauri::command]
#[specta::specta]
pub fn get_damage_row(state: tauri::State<'_, EncounterMutex>) -> DPSWindow {
    let encounter = state.lock().unwrap();

    let mut dps_window = DPSWindow {
        dps_rows: DPSRows::default(),
        ..Default::default()
    };

    let time_elapsed_ms = encounter.time_last_combat_packet.saturating_sub(encounter.time_fight_start);
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    for (&entity_uid, entity) in &encounter.entity_uid_to_entity { // calculate things like dps
        let is_player = entity.entity_type == EEntityType::EntChar;
        let did_damage = !entity.skill_uid_to_skill.is_empty();
        // info!("{}, {is_player}", entity.name);
        if is_player && did_damage {
            // Damage Stats per player
            let mut damage_row = DPSRow {
                uid: entity_uid,
                name:
                if entity_uid == encounter.local_player_uid && entity.name.is_empty() {
                    String::from("You")
                } else if entity_uid == encounter.local_player_uid && !entity.name.is_empty() {
                    format!("{} (You)", entity.name)
                } else {
                    entity.name.clone()
                },
                class: class::to_string(entity.class_id),
                // class_style: String::new(),
                ability_score: entity.ability_score,
                total_dmg: entity.total_dmg,
                dps: entity.total_dmg as f64 / time_elapsed_secs.max(1.0),
                crit_rate: entity.crit_hits as f64 / entity.hits as f64 * 100.0,
                crit_dmg_rate: entity.crit_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
                lucky_rate: entity.lucky_hits as f64 / entity.hits as f64 * 100.0,
                lucky_dmg_rate: entity.lucky_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
                hits: entity.hits,
                hits_per_second: entity.hits as f64 / time_elapsed_secs.max(1.0),
                ..Default::default()
            };
            dps_window.total_dmg += entity.total_dmg;
            dps_window.dps_rows.push(damage_row);

        }
    }

    drop(encounter); // todo: is this a good idea? dropping before expensive sort

    dps_window.elapsed_ms = time_elapsed_ms;
    dps_window.dps_rows.sort_by(|this_row, other_row| { // sort descending by dps
        other_row.total_dmg
            .partial_cmp(&this_row.total_dmg) // descending
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    dps_window
}

#[tauri::command]
#[specta::specta]
pub fn get_skill_row(state: tauri::State<'_, EncounterMutex>, player_uid_str: String) -> DPSSkillBreakdownWindow {
    let player_uid: i64 = player_uid_str.parse().unwrap();
    let encounter = state.lock().unwrap();
    // todo: If lookup fails, send smth where we check in the UI if it fails, just go back 1 route
    let entity = encounter.entity_uid_to_entity.get(&player_uid).unwrap();

    let time_elapsed_secs = encounter.time_last_combat_packet.saturating_sub(encounter.time_fight_start) as f64 / 1000.0;

    let mut skill_window = DPSSkillBreakdownWindow::default();
    // Player DPS Stats
    skill_window.curr_player = DPSRow {
        uid: player_uid,
        name:
        if player_uid == encounter.local_player_uid && entity.name.is_empty() {
            String::from("You")
        } else if player_uid == encounter.local_player_uid && !entity.name.is_empty() {
            format!("{} (You)", entity.name)
        } else {
            entity.name.clone()
        },
        class: class::to_string(entity.class_id),
        ability_score: entity.ability_score,
        total_dmg: entity.total_dmg,
        ..Default::default()
    };

    // Skills for this player
    for (&skill_uid, skill) in &entity.skill_uid_to_skill {
        let skill_row = SkillRow {
            name: Skill::get_skill_name(skill_uid),
            total_dmg: skill.total_dmg,
            dps: (skill.total_dmg as f64) / time_elapsed_secs.max(1.0) * 100.0,
            hits_per_second: skill.hits as f64 / time_elapsed_secs.max(1.0) * 100.0,
            crit_rate: skill.crit_hits as f64 / skill.hits as f64 * 100.0,
            crit_dmg_rate: skill.crit_total_dmg as f64 / skill.total_dmg as f64 * 100.0,
            lucky_rate: skill.lucky_hits as f64 / skill.hits as f64 * 100.0,
            lucky_dmg_rate: skill.lucky_total_dmg as f64 / skill.total_dmg as f64 * 100.0,
            hits: skill.hits,
            ..Default::default()
        };
        skill_window.skill_rows.push(skill_row);
    }

    drop(encounter);

    // Sort skills descending by DPS
    skill_window.skill_rows.sort_by(|this_row, other_row| {
        other_row.total_dmg
            .partial_cmp(&this_row.total_dmg) // descending
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    skill_window
}
