use crate::live::commands_models::{DamageRow, DamageRows};
use crate::live::opcodes_models::{class, EncounterMutex};
use blueprotobuf_lib::blueprotobuf::EEntityType;

#[tauri::command]
#[specta::specta]
pub fn get_damage_row(state: tauri::State<'_, EncounterMutex>) -> DamageRows {
    let encounter = state.lock().unwrap();

    let mut damage_display = DamageRows::new();
    for (&entity_uid, entity) in &encounter.uid_to_entity { // calculate things like dps
        let is_player = entity.entity_type == EEntityType::EntChar;
        let did_damage = entity.damage_stats.damage_dealt > 0;
        if is_player && did_damage {
            let time_elapsed_secs = (encounter.time_last_combat_packet.saturating_sub(encounter.time_fight_start) as f64) / 1000.0;
            let damage_row = DamageRow {
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

            damage_display.push(damage_row);
        }
    }
    damage_display.sort_by(|this_row, other_row| { // sort descending by dps
        other_row.dps
            .partial_cmp(&this_row.dps) // descending
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    damage_display
}
