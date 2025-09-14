use std::default::Default;
use std::fmt::format;
use crate::live::models::{DamageStats, Encounter, EncounterInner, Entity};
use blueprotobuf_lib::blueprotobuf;
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::info;
use tauri::{AppHandle, Manager};

pub fn on_server_change(app_handle: &AppHandle,) {
    info!("on server change");
    app_handle
        .state::<Encounter>()
        .lock()
        .unwrap()
        .clone_from(&EncounterInner::default());
}

pub fn process_sync_to_me_delta_info(
    app_handle: &AppHandle,
    sync_to_me_delta_info: blueprotobuf::SyncToMeDeltaInfo,
) -> Option<()>{
    let delta_info = sync_to_me_delta_info.delta_info?;
    let my_uuid = delta_info.uuid; // UUID =/= uid (have to >> 16) // todo: add my UID here
    process_aoi_sync_delta(app_handle, delta_info.base_delta?);
    Some(())
}

pub fn process_aoi_sync_delta(
    app_handle: &AppHandle,
    aoi_sync_delta: blueprotobuf::AoiSyncDelta,
) -> Option<()> {

    let encounter_state = app_handle.state::<Encounter>();
    let mut encounter_state = encounter_state.lock().unwrap();

    let target_uuid = aoi_sync_delta.uuid?; // UUID =/= uid (have to >> 16)
    let mut target_name = String::new();
    for attr in aoi_sync_delta.attrs?.attrs { // todo: add function to process separately
        match attr.id? {
            0x01 => {
                target_name = String::from_utf8_lossy(&(attr.raw_data?)).to_string();
            }
            _ => {}
        }
    }

    let target_entity = encounter_state.entities.entry(target_uuid)
        .or_insert_with(|| Entity {
            uid: target_uuid >> 16,
            name: target_name,
            entity_type: EEntityType::from(target_uuid),
            ..Default::default()
        }).to_owned();

    for sync_damage_info in aoi_sync_delta.skill_effects?.damages {
        let non_lucky_dmg = sync_damage_info.value;
        let lucky_dmg = sync_damage_info.lucky_value;

        // Check dmg then lucky dmg todo: confused, why is lucky dmg less priority?
        let actual_dmg: i64 = match non_lucky_dmg.or(lucky_dmg) {
            Some(actual_dmg) => actual_dmg,
            None => continue, // skip this iteration
        };

        let attacker_uuid = sync_damage_info
            .top_summoner_id
            .or(sync_damage_info.attacker_uuid)?;


        let attacker_entity = encounter_state.entities.entry(attacker_uuid)
            .or_insert_with(|| Entity {
                uid: attacker_uuid >> 16,
                name: format!("dummy-{attacker_uuid}-name"),
                entity_type: EEntityType::from(attacker_uuid),
                damage_stats: DamageStats {
                    ..Default::default()
                },
                ..Default::default()
            });

        attacker_entity.damage_stats.damage_dealt += actual_dmg;
        info!("{} to {}: {actual_dmg} dmg {} total dmg", attacker_uuid >> 16, target_uuid >> 16, attacker_entity.damage_stats.damage_dealt);
    }
    Some(())
}
