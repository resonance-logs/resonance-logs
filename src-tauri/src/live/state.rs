use crate::database::{DbTask, enqueue, now_ms};
use crate::live::event_manager::{EventManager, MetricType};
use crate::live::opcodes_models::Encounter;
use crate::live::player_names::PlayerNames;
use blueprotobuf_lib::blueprotobuf;
use log::{info, warn};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum StateEvent {
    ServerChange,
    EnterScene(blueprotobuf::EnterScene),
    SyncNearEntities(blueprotobuf::SyncNearEntities),
    SyncContainerData(blueprotobuf::SyncContainerData),
    SyncContainerDirtyData(blueprotobuf::SyncContainerDirtyData),
    SyncServerTime(blueprotobuf::SyncServerTime),
    SyncToMeDeltaInfo(blueprotobuf::SyncToMeDeltaInfo),
    SyncNearDeltaInfo(blueprotobuf::SyncNearDeltaInfo),
    PauseEncounter(bool),
    ResetEncounter,
}

#[derive(Debug)]
pub struct AppState {
    pub encounter: Encounter,
    pub event_manager: EventManager,
    pub skill_subscriptions: HashSet<(i64, String)>,
    pub app_handle: AppHandle,
    pub boss_only_dps: bool,
    pub low_hp_bosses: HashMap<i64, u128>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            encounter: Encounter::default(),
            event_manager: EventManager::new(),
            skill_subscriptions: HashSet::new(),
            app_handle,
            boss_only_dps: false,
            low_hp_bosses: HashMap::new(),
        }
    }

    pub fn is_encounter_paused(&self) -> bool {
        self.encounter.is_encounter_paused
    }

    pub fn set_encounter_paused(&mut self, paused: bool) {
        self.encounter.is_encounter_paused = paused;
        self.event_manager.emit_encounter_pause(paused);
    }
}

#[derive(Clone)]
pub struct AppStateManager {
    pub state: Arc<RwLock<AppState>>,
}

impl AppStateManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let state = AppState::new(app_handle);
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub async fn handle_event(&self, event: StateEvent) {
        let mut state = self.state.write().await;

        // Check if encounter is paused for events that should be dropped
        if state.is_encounter_paused()
            && matches!(
                event,
                StateEvent::SyncNearEntities(_)
                    | StateEvent::SyncContainerData(_)
                    | StateEvent::SyncContainerDirtyData(_)
                    | StateEvent::SyncToMeDeltaInfo(_)
                    | StateEvent::SyncNearDeltaInfo(_)
            )
        {
            info!("packet dropped due to encounter paused");
            return;
        }

        match event {
            StateEvent::ServerChange => {
                self.on_server_change(&mut state).await;
            }
            StateEvent::EnterScene(data) => {
                self.process_enter_scene(&mut state, data).await;
            }
            StateEvent::SyncNearEntities(data) => {
                self.process_sync_near_entities(&mut state, data).await;
                // Note: Player names are automatically stored in the database via UpsertEntity tasks
                // No need to maintain a separate cache anymore
            }
            StateEvent::SyncContainerData(data) => {
                // store local_player copy
                state.encounter.local_player = data.clone();

                self.process_sync_container_data(&mut state, data).await;
                // Note: Player names are automatically stored in the database via UpsertEntity tasks
                // No need to maintain a separate cache anymore
            }
            StateEvent::SyncContainerDirtyData(data) => {
                self.process_sync_container_dirty_data(&mut state, data)
                    .await;
            }
            StateEvent::SyncServerTime(_data) => {
                // todo: this is skipped, not sure what info it has
            }
            StateEvent::SyncToMeDeltaInfo(data) => {
                self.process_sync_to_me_delta_info(&mut state, data).await;
                // Note: Player names are automatically stored in the database via UpsertEntity tasks
                // No need to maintain a separate cache anymore
            }
            StateEvent::SyncNearDeltaInfo(data) => {
                self.process_sync_near_delta_info(&mut state, data).await;
                // Note: Player names are automatically stored in the database via UpsertEntity tasks
                // No need to maintain a separate cache anymore
            }
            StateEvent::PauseEncounter(paused) => {
                state.set_encounter_paused(paused);
            }
            StateEvent::ResetEncounter => {
                self.reset_encounter(&mut state).await;
            }
        }
    }

    async fn on_server_change(&self, state: &mut AppState) {
        use crate::live::opcodes_process::on_server_change;
        // End any active encounter in DB
        enqueue(DbTask::EndEncounter {
            ended_at_ms: now_ms(),
        });
        on_server_change(&mut state.encounter);

        // Emit encounter reset event
        if state.event_manager.should_emit_events() {
            state.event_manager.emit_encounter_reset();
            // Clear dead bosses tracking on server change
            state.event_manager.clear_dead_bosses();
        }

        // Clear skill subscriptions
        state.skill_subscriptions.clear();
        state.low_hp_bosses.clear();
    }
        // all scene id extraction logic is here (its pretty rough)
    async fn process_enter_scene(
        &self,
        state: &mut AppState,
        enter_scene: blueprotobuf::EnterScene,
    ) {
        use crate::live::scene_names;

        info!("EnterScene packet received");

        // Quick check: if a scene_guid string is present, try to parse digits from it
        if let Some(info) = enter_scene.enter_scene_info.as_ref() {
            if let Some(guid) = &info.scene_guid {
                info!("EnterScene.scene_guid present: {}", guid);
                // Try to extract numeric part of the guid
                let digits: String = guid.chars().filter(|c| c.is_ascii_digit()).collect();
                if !digits.is_empty() {
                    if let Ok(v) = digits.parse::<i32>() {
                        if scene_names::contains(v) {
                            info!("Parsed scene id {} from scene_guid", v);
                            // Directly use this id
                            let name = scene_names::lookup(v);
                            state.encounter.current_scene_id = Some(v);
                            state.encounter.current_scene_name = Some(name.clone());
                            if state.event_manager.should_emit_events() {
                                state.event_manager.emit_scene_change(name);
                            }
                            return;
                        }
                    }
                }
            }
            if let Some(connect) = &info.connect_guid {
                info!("EnterScene.connect_guid present: {}", connect);
            }
        }

        // Helper: try to find a known scene id by scanning varints at every offset
        let find_scene_id_in_bytes = |data: &[u8]| -> Option<i32> {
            // 1) Try protobuf varint decoding at every offset
            for offset in 0..data.len() {
                let mut slice = &data[offset..];
                if let Ok(v) = prost::encoding::decode_varint(&mut slice) {
                    if v <= i32::MAX as u64 {
                        let cand = v as i32;
                        if scene_names::contains(cand) {
                            return Some(cand);
                        }
                    }
                }
            }

            // 2) Try 4-byte little-endian and big-endian ints
            if data.len() >= 4 {
                for i in 0..=data.len() - 4 {
                    let le = i32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
                    if le > 0 && scene_names::contains(le) {
                        return Some(le);
                    }
                    let be = i32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
                    if be > 0 && scene_names::contains(be) {
                        return Some(be);
                    }
                }
            }

            // 3) Try ASCII decimal substrings of length 2..6
            let mut i = 0;
            while i < data.len() {
                if data[i].is_ascii_digit() {
                    let start = i;
                    i += 1;
                    while i < data.len() && data[i].is_ascii_digit() {
                        i += 1;
                    }
                    let len_digits = i - start;
                    if len_digits >= 2 && len_digits <= 6 {
                        if let Ok(s) = std::str::from_utf8(&data[start..i]) {
                            if let Ok(v) = s.parse::<i32>() {
                                if scene_names::contains(v) {
                                    return Some(v);
                                }
                            }
                        }
                    }
                } else {
                    i += 1;
                }
            }

            None
        };

        // Try several likely locations in the EnterSceneInfo where a scene id might be present
        let mut found_scene: Option<i32> = None;
        if let Some(info) = enter_scene.enter_scene_info.as_ref() {
            // 1) Inspect explicit attr collections (subscene_attrs then scene_attrs)
            for maybe_attrs in [info.subscene_attrs.as_ref(), info.scene_attrs.as_ref()] {
                if let Some(attrs) = maybe_attrs {
                    // Check simple attrs (varint or length-prefixed)
                    for attr in &attrs.attrs {
                        if found_scene.is_some() {
                            break;
                        }
                        if let Some(raw) = &attr.raw_data {
                            // If attr id suggests a scene id, prefer that first
                            if let Some(attr_id) = attr.id {
                                if attr_id == 0x01 || attr_id == 0x0a {
                                    let mut buf = &raw[..];
                                    if let Ok(v) = prost::encoding::decode_varint(&mut buf) {
                                        let cand = v as i32;
                                        if scene_names::contains(cand) {
                                            info!("Found scene_id {} in attr {}", cand, attr_id);
                                            found_scene = Some(cand);
                                            break;
                                        }
                                    }
                                }
                            }

                            // Fallback: scan all varints in the raw bytes for a known scene id
                            if found_scene.is_none() {
                                if let Some(cand) = find_scene_id_in_bytes(raw) {
                                    info!("Found scene_id {} by scanning attr raw bytes", cand);
                                    found_scene = Some(cand);
                                    break;
                                }
                            }
                        }
                    }

                    if found_scene.is_some() {
                        break;
                    }

                    // Check map_attrs entries (keys/values may contain the id)
                    for map_attr in &attrs.map_attrs {
                        if found_scene.is_some() {
                            break;
                        }
                        for kv in &map_attr.attrs {
                            if found_scene.is_some() {
                                break;
                            }
                            if let Some(val) = &kv.value {
                                if let Some(cand) = find_scene_id_in_bytes(val) {
                                    info!("Found scene_id {} in map_attr value (map id {:?})", cand, map_attr.id);
                                    found_scene = Some(cand);
                                    break;
                                }
                            }
                            if let Some(key) = &kv.key {
                                if let Some(cand) = find_scene_id_in_bytes(key) {
                                    info!("Found scene_id {} in map_attr key (map id {:?})", cand, map_attr.id);
                                    found_scene = Some(cand);
                                    break;
                                }
                            }
                        }
                    }

                    if found_scene.is_some() {
                        break;
                    }
                }
            }

            // 2) As a fallback, inspect player_ent.attrs if present
            if found_scene.is_none() {
                if let Some(player_ent) = &info.player_ent {
                    if let Some(player_attrs) = &player_ent.attrs {
                        for attr in &player_attrs.attrs {
                            if let Some(raw) = &attr.raw_data {
                                if let Some(cand) = find_scene_id_in_bytes(raw) {
                                    info!("Found scene_id {} in player_ent attrs", cand);
                                    found_scene = Some(cand);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(scene_id) = found_scene {
            let scene_name = scene_names::lookup(scene_id);

            // Update encounter with scene info
            state.encounter.current_scene_id = Some(scene_id);
            state.encounter.current_scene_name = Some(scene_name.clone());

            info!("Scene changed to: {} (ID: {})", scene_name, scene_id);

            // Emit scene change event
            if state.event_manager.should_emit_events() {
                info!("Emitting scene change event for: {}", scene_name);
                state.event_manager.emit_scene_change(scene_name.clone());
            } else {
                warn!("Event manager not ready, skipping scene change emit");
            }
        } else {
            warn!("Could not extract scene_id from EnterScene packet - dumping attrs for debugging");

            // Helper to produce a short hex snippet for binary data
            let to_hex_snip = |data: &[u8]| -> String {
                data.iter()
                    .take(16)
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join("")
            };

            if let Some(info) = enter_scene.enter_scene_info.as_ref() {
                for (label, maybe_attrs) in [("subscene_attrs", info.subscene_attrs.as_ref()), ("scene_attrs", info.scene_attrs.as_ref())] {
                    if let Some(attrs) = maybe_attrs {
                        info!("Inspecting {}: uuid={:?}, #attrs={}, #map_attrs={}", label, attrs.uuid, attrs.attrs.len(), attrs.map_attrs.len());

                        for attr in &attrs.attrs {
                            let id = attr.id.unwrap_or(-1);
                            let len = attr.raw_data.as_ref().map(|b| b.len()).unwrap_or(0);
                            let snip = attr.raw_data.as_ref().map(|b| to_hex_snip(b)).unwrap_or_default();
                            info!("  attr id={} len={} snippet={}", id, len, snip);
                        }

                        for map_attr in &attrs.map_attrs {
                            info!("  map_attr id={:?} #entries={}", map_attr.id, map_attr.attrs.len());
                            for kv in &map_attr.attrs {
                                let key_len = kv.key.as_ref().map(|k| k.len()).unwrap_or(0);
                                let val_len = kv.value.as_ref().map(|v| v.len()).unwrap_or(0);
                                let key_snip = kv.key.as_ref().map(|k| to_hex_snip(k)).unwrap_or_default();
                                let val_snip = kv.value.as_ref().map(|v| to_hex_snip(v)).unwrap_or_default();
                                info!("    entry key_len={} val_len={} key_snip={} val_snip={}", key_len, val_len, key_snip, val_snip);
                            }
                        }
                    }
                }

                if let Some(player_ent) = &info.player_ent {
                    if let Some(player_attrs) = &player_ent.attrs {
                        info!("Inspecting player_ent.attrs: #attrs={}", player_attrs.attrs.len());
                        for attr in &player_attrs.attrs {
                            let id = attr.id.unwrap_or(-1);
                            let len = attr.raw_data.as_ref().map(|b| b.len()).unwrap_or(0);
                            let snip = attr.raw_data.as_ref().map(|b| to_hex_snip(b)).unwrap_or_default();
                            info!("  player attr id={} len={} snippet={}", id, len, snip);
                        }
                    }
                }
            }

            // Emit a fallback scene change event so frontend still notifies the user
            let fallback_name = enter_scene
                .enter_scene_info
                .as_ref()
                .and_then(|i| i.scene_guid.clone())
                .map(|g| format!("Scene GUID: {}", g))
                .unwrap_or_else(|| "Unknown Scene".to_string());

            state.encounter.current_scene_name = Some(fallback_name.clone());
            if state.event_manager.should_emit_events() {
                info!("Emitting fallback scene change event: {}", fallback_name);
                state.event_manager.emit_scene_change(fallback_name);
            }
        }
    }

    async fn process_sync_near_entities(
        &self,
        state: &mut AppState,
        sync_near_entities: blueprotobuf::SyncNearEntities,
    ) {
        use crate::live::opcodes_process::process_sync_near_entities;
        if process_sync_near_entities(&mut state.encounter, sync_near_entities).is_none() {
            warn!("Error processing SyncNearEntities.. ignoring.");
        }
    }

    async fn process_sync_container_data(
        &self,
        state: &mut AppState,
        sync_container_data: blueprotobuf::SyncContainerData,
    ) {
        use crate::live::opcodes_process::process_sync_container_data;
        if process_sync_container_data(&mut state.encounter, sync_container_data).is_none() {
            warn!("Error processing SyncContainerData.. ignoring.");
        }
    }

    async fn process_sync_container_dirty_data(
        &self,
        state: &mut AppState,
        sync_container_dirty_data: blueprotobuf::SyncContainerDirtyData,
    ) {
        use crate::live::opcodes_process::process_sync_container_dirty_data;
        if process_sync_container_dirty_data(&mut state.encounter, sync_container_dirty_data)
            .is_none()
        {
            warn!("Error processing SyncContainerDirtyData.. ignoring.");
        }
    }

    async fn process_sync_to_me_delta_info(
        &self,
        state: &mut AppState,
        sync_to_me_delta_info: blueprotobuf::SyncToMeDeltaInfo,
    ) {
        use crate::live::opcodes_process::process_sync_to_me_delta_info;
        // Missing fields are normal, no need to log
        let _ = process_sync_to_me_delta_info(&mut state.encounter, sync_to_me_delta_info);
    }

    async fn process_sync_near_delta_info(
        &self,
        state: &mut AppState,
        sync_near_delta_info: blueprotobuf::SyncNearDeltaInfo,
    ) {
        use crate::live::opcodes_process::process_aoi_sync_delta;
        for aoi_sync_delta in sync_near_delta_info.delta_infos {
            // Missing fields are normal, no need to log
            let _ = process_aoi_sync_delta(&mut state.encounter, aoi_sync_delta);
        }
    }

    async fn reset_encounter(&self, state: &mut AppState) {
        // End any active encounter in DB
        enqueue(DbTask::EndEncounter {
            ended_at_ms: now_ms(),
        });
        state.encounter.reset_combat_state();
        state.skill_subscriptions.clear();

        if state.event_manager.should_emit_events() {
            state.event_manager.emit_encounter_reset();
            // Clear dead bosses tracking on reset
            state.event_manager.clear_dead_bosses();
        }

        state.low_hp_bosses.clear();
        state.skill_subscriptions.clear();
    }

    pub async fn get_encounter(&self) -> Encounter {
        self.state.read().await.encounter.clone()
    }

    // Manage skill subscriptions
    pub async fn update_skill_subscriptions<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut HashSet<(i64, String)>),
    {
        let mut state = self.state.write().await;
        update_fn(&mut state.skill_subscriptions);
    }

    /// Get player name by UID from database
    pub async fn get_player_name(&self, uid: i64) -> Option<String> {
        PlayerNames::get_name_by_uid(uid)
    }

    /// Get recent players ordered by last seen (most recent first)
    pub async fn get_recent_players(&self, limit: usize) -> Vec<(i64, String)> {
        PlayerNames::get_recent_players(limit)
    }

    /// Get multiple names by UIDs (batch query for performance)
    pub async fn get_player_names(&self, uids: &[i64]) -> std::collections::HashMap<i64, String> {
        PlayerNames::get_names_by_uids(uids)
    }

    /// Check if a player exists in the database
    pub async fn contains_player(&self, uid: i64) -> bool {
        PlayerNames::contains_player(uid)
    }

    pub async fn update_encounter<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut Encounter),
    {
        let mut state = self.state.write().await;
        update_fn(&mut state.encounter);
    }

    pub async fn get_encounter_ref(&self) -> tokio::sync::RwLockReadGuard<'_, AppState> {
        self.state.read().await
    }

    pub async fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&AppState) -> R,
    {
        let state = self.state.read().await;
        f(&*state)
    }

    pub async fn with_state_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut AppState) -> R,
    {
        let mut state = self.state.write().await;
        f(&mut *state)
    }
}

impl AppStateManager {
    pub async fn update_and_emit_events(&self) {
        // First, read the encounter data to generate all the necessary information
        let (encounter, should_emit, boss_only) = {
            let state = self.state.read().await;
            (
                state.encounter.clone(),
                state.event_manager.should_emit_events(),
                state.boss_only_dps,
            )
        };

        if !should_emit {
            return;
        }

        // Generate all the data we need without holding the lock
        let header_info_with_deaths = crate::live::event_manager::generate_header_info(&encounter, boss_only);
        let dps_players =
            crate::live::event_manager::generate_players_window_dps(&encounter, boss_only);
        let heal_players = crate::live::event_manager::generate_players_window_heal(&encounter);
        let tanked_players = crate::live::event_manager::generate_players_window_tanked(&encounter);

        // Generate skill windows for all players
        let mut dps_skill_windows = Vec::new();
        let mut heal_skill_windows = Vec::new();
        let mut tanked_skill_windows = Vec::new();
        let mut subscribed_players = Vec::new();

        for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
            let is_player = entity.entity_type == blueprotobuf::EEntityType::EntChar;
            let has_dmg_skills = !entity.skill_uid_to_dmg_skill.is_empty();
            let has_heal_skills = !entity.skill_uid_to_heal_skill.is_empty();
            let has_taken_skills = !entity.skill_uid_to_taken_skill.is_empty();

            if is_player && has_dmg_skills {
                if let Some(skills_window) = crate::live::event_manager::generate_skills_window_dps(
                    &encounter, entity_uid, boss_only,
                ) {
                    dps_skill_windows.push((entity_uid, skills_window));
                }
            }

            if is_player && has_heal_skills {
                if let Some(skills_window) =
                    crate::live::event_manager::generate_skills_window_heal(&encounter, entity_uid)
                {
                    heal_skill_windows.push((entity_uid, skills_window));
                }
            }

            if is_player && has_taken_skills {
                if let Some(skills_window) =
                    crate::live::event_manager::generate_skills_window_tanked(
                        &encounter, entity_uid,
                    )
                {
                    tanked_skill_windows.push((entity_uid, skills_window));
                }
            }

            // Collect subscribed players for later emission
            subscribed_players.push(entity_uid);
        }

        // Now, acquire the write lock and update everything
        let mut state = self.state.write().await;

        // Emit encounter update and handle boss deaths
        if let Some((mut header_info, mut dead_bosses)) = header_info_with_deaths {
            use std::collections::HashSet;

            let mut dead_ids: HashSet<i64> = dead_bosses.iter().map(|(uid, _)| *uid).collect();
            let current_time_ms = now_ms() as u128;

            for boss in &mut header_info.bosses {
                let hp_percent = if let (Some(curr_hp), Some(max_hp)) = (boss.current_hp, boss.max_hp) {
                    if max_hp > 0 {
                        curr_hp as f64 / max_hp as f64 * 100.0
                    } else {
                        0.0
                    }
                } else {
                    100.0
                };

                if hp_percent < 5.0 {
                    let entry = state.low_hp_bosses.entry(boss.uid).or_insert(current_time_ms);
                    if current_time_ms.saturating_sub(*entry) >= 5_000 {
                        if dead_ids.insert(boss.uid) {
                            dead_bosses.push((boss.uid, boss.name.clone()));
                        }
                    }
                } else {
                    state.low_hp_bosses.remove(&boss.uid);
                }

                if dead_ids.contains(&boss.uid) {
                    boss.current_hp = Some(0);
                    state.low_hp_bosses.remove(&boss.uid);
                }
            }

            state.event_manager.emit_encounter_update(
                header_info,
                encounter.is_encounter_paused, // Use the original encounter state
            );

            // Emit boss death events for newly dead bosses
            for (boss_uid, boss_name) in dead_bosses {
                state.event_manager.emit_boss_death(boss_name, boss_uid);
            }
        }

        // Emit DPS players update
        if !dps_players.player_rows.is_empty() {
            state
                .event_manager
                .emit_players_update(MetricType::Dps, dps_players);
        }

        // Emit heal players update
        if !heal_players.player_rows.is_empty() {
            state
                .event_manager
                .emit_players_update(MetricType::Heal, heal_players);
        }

        // Emit tanked players update
        if !tanked_players.player_rows.is_empty() {
            state
                .event_manager
                .emit_players_update(MetricType::Tanked, tanked_players);
        }

        // Emit skills updates only for subscribed players using precomputed windows
        for (entity_uid, skills_window) in &dps_skill_windows {
            if state
                .skill_subscriptions
                .contains(&(*entity_uid, "dps".to_string()))
            {
                state.event_manager.emit_skills_update(
                    MetricType::Dps,
                    *entity_uid,
                    skills_window.clone(),
                );
            }
        }
        for (entity_uid, skills_window) in &heal_skill_windows {
            if state
                .skill_subscriptions
                .contains(&(*entity_uid, "heal".to_string()))
            {
                state.event_manager.emit_skills_update(
                    MetricType::Heal,
                    *entity_uid,
                    skills_window.clone(),
                );
            }
        }
        for (entity_uid, skills_window) in &tanked_skill_windows {
            if state
                .skill_subscriptions
                .contains(&(*entity_uid, "tanked".to_string()))
            {
                state.event_manager.emit_skills_update(
                    MetricType::Tanked,
                    *entity_uid,
                    skills_window.clone(),
                );
            }
        }
    }
}
