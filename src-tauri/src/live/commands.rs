use std::collections::HashMap;
use crate::live::commands_models::{DPSRow, DPSRows, DPSWindow, SkillRow};
use crate::live::opcodes_models::{class, EncounterMutex, Skill};
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::info;
use once_cell::sync::Lazy;

#[tauri::command]
#[specta::specta]
pub fn get_damage_row(state: tauri::State<'_, EncounterMutex>) -> DPSWindow {
    let encounter = state.lock().unwrap();

    let mut dps_window = DPSWindow {
        damage_rows: DPSRows::default(),
        ..Default::default()
    };

    for (&entity_uid, entity) in &encounter.entity_uid_to_entity { // calculate things like dps
        let is_player = entity.entity_type == EEntityType::EntChar;
        let did_damage = entity.damage_stats.damage_dealt >= 0; // todo: fix later > 0
        // info!("{}, {is_player}", entity.name);
        if is_player && did_damage {
            let time_elapsed_secs = (encounter.time_last_combat_packet.saturating_sub(encounter.time_fight_start) as f64) / 1000.0;
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
                total_damage: entity.damage_stats.damage_dealt,
                #[allow(clippy::cast_precision_loss)]
                dps: (entity.damage_stats.damage_dealt as f64) / time_elapsed_secs,
                ..Default::default()
            };

            for (&skill_uid, skill) in &entity.skill_uid_to_skill {
                let skill_row = SkillRow {
                    name: Skill::get_skill_name(skill_uid),
                    total_damage: skill.total_damage,
                    dps: (skill.total_damage as f64) / time_elapsed_secs,
                    ..Default::default()
                };
                damage_row.skills.push(skill_row);
            }
            damage_row.skills.sort_by(|this_row, other_row| { // sort descending by dps
                other_row.total_damage
                    .partial_cmp(&this_row.total_damage) // descending
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            dps_window.damage_rows.push(damage_row);
            dps_window.total_damage += entity.damage_stats.damage_dealt;
        }
    }
    dps_window.damage_rows.sort_by(|this_row, other_row| { // sort descending by dps
        other_row.total_damage
            .partial_cmp(&this_row.total_damage) // descending
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    dps_window
}
