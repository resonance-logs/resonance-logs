use crate::live::opcodes_models::Encounter;
use crate::live::name_cache::RecentNamesCache;
use crate::live::event_manager::{EventManager, MetricType};
use crate::live::skills_store::SkillsStore;
use blueprotobuf_lib::blueprotobuf;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::AppHandle;
use crate::database::{enqueue, DbTask, now_ms};

#[derive(Debug, Clone)]
pub enum StateEvent {
    ServerChange,
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
    pub skills_store: SkillsStore,
    pub app_handle: AppHandle,
    pub recent_names: RecentNamesCache,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        // Attempt to read capacity from capabilities live.json, fallback to default
        let cap = RecentNamesCache::capacity_from_capabilities();
        Self {
            encounter: Encounter::default(),
            event_manager: EventManager::new(),
            skills_store: SkillsStore::new(),
            app_handle,
            recent_names: RecentNamesCache::with_capacity(cap),
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
        if state.is_encounter_paused() && matches!(event,
            StateEvent::SyncNearEntities(_) |
            StateEvent::SyncContainerData(_) |
            StateEvent::SyncContainerDirtyData(_) |
            StateEvent::SyncToMeDeltaInfo(_) |
            StateEvent::SyncNearDeltaInfo(_)
        ) {
            info!("packet dropped due to encounter paused");
            return;
        }

        match event {
            StateEvent::ServerChange => {
                self.on_server_change(&mut state).await;
            }
            StateEvent::SyncNearEntities(data) => {
                // Collect UIDs present in this packet so we can update recent_names
                let uids: Vec<i64> = data
                    .appear
                    .iter()
                    .filter_map(|pkt_entity| pkt_entity.uuid)
                    .map(|uuid| uuid >> 16)
                    .collect();

                self.process_sync_near_entities(&mut state, data).await;

                // Update cache for any names discovered
                for uid in uids {
                    let maybe_name = if let Some(entity) = state.encounter.entity_uid_to_entity.get(&uid) {
                        Some(entity.name.clone())
                    } else {
                        None
                    };

                    if let Some(name) = maybe_name {
                        if !name.is_empty() {
                            state.recent_names.add_name(uid, name);
                        }
                    }
                }
            }
            StateEvent::SyncContainerData(data) => {
                // store local_player copy
                state.encounter.local_player = data.clone();

                // extract player uid (if available) so we can update cache after processing
                let player_uid_opt = data
                    .v_data
                    .as_ref()
                    .and_then(|v| v.char_id);

                self.process_sync_container_data(&mut state, data).await;

                if let Some(player_uid) = player_uid_opt {
                    let maybe_name = if let Some(entity) = state.encounter.entity_uid_to_entity.get(&player_uid) {
                        Some(entity.name.clone())
                    } else {
                        None
                    };
                    if let Some(name) = maybe_name {
                        if !name.is_empty() {
                            state.recent_names.add_name(player_uid, name);
                        }
                    }
                }
            }
            StateEvent::SyncContainerDirtyData(data) => {
                self.process_sync_container_dirty_data(&mut state, data).await;
            }
            StateEvent::SyncServerTime(_data) => {
                // todo: this is skipped, not sure what info it has
            }
            StateEvent::SyncToMeDeltaInfo(data) => {
                // delta_info may contain a single entity's uid
                let uid_opt = data.delta_info.as_ref().and_then(|d| d.uuid).map(|u| u >> 16);
                self.process_sync_to_me_delta_info(&mut state, data).await;
                if let Some(uid) = uid_opt {
                    let maybe_name = if let Some(entity) = state.encounter.entity_uid_to_entity.get(&uid) {
                        Some(entity.name.clone())
                    } else {
                        None
                    };
                    if let Some(name) = maybe_name {
                        if !name.is_empty() {
                            state.recent_names.add_name(uid, name);
                        }
                    }
                }
            }
            StateEvent::SyncNearDeltaInfo(data) => {
                // collect uids from delta infos
                let uids: Vec<i64> = data
                    .delta_infos
                    .iter()
                    .filter_map(|d| d.uuid)
                    .map(|uuid| uuid >> 16)
                    .collect();

                self.process_sync_near_delta_info(&mut state, data).await;

                for uid in uids {
                    let maybe_name = if let Some(entity) = state.encounter.entity_uid_to_entity.get(&uid) {
                        Some(entity.name.clone())
                    } else {
                        None
                    };
                    if let Some(name) = maybe_name {
                        if !name.is_empty() {
                            state.recent_names.add_name(uid, name);
                        }
                    }
                }
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
        enqueue(DbTask::EndEncounter { ended_at_ms: now_ms() });
        on_server_change(&mut state.encounter);

        // Emit encounter reset event
        if state.event_manager.should_emit_events() {
            state.event_manager.emit_encounter_reset();
        }

        // Clear skills store
        state.skills_store.clear();
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
        if process_sync_container_dirty_data(&mut state.encounter, sync_container_dirty_data).is_none() {
            warn!("Error processing SyncContainerDirtyData.. ignoring.");
        }
    }

    async fn process_sync_to_me_delta_info(
        &self,
        state: &mut AppState,
        sync_to_me_delta_info: blueprotobuf::SyncToMeDeltaInfo,
    ) {
        use crate::live::opcodes_process::process_sync_to_me_delta_info;
        if process_sync_to_me_delta_info(&mut state.encounter, sync_to_me_delta_info).is_none() {
            warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
        }
    }

    async fn process_sync_near_delta_info(
        &self,
        state: &mut AppState,
        sync_near_delta_info: blueprotobuf::SyncNearDeltaInfo,
    ) {
        use crate::live::opcodes_process::process_aoi_sync_delta;
        for aoi_sync_delta in sync_near_delta_info.delta_infos {
            if process_aoi_sync_delta(&mut state.encounter, aoi_sync_delta).is_none() {
                warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
            }
        }
    }

    async fn reset_encounter(&self, state: &mut AppState) {
        // End any active encounter in DB
        enqueue(DbTask::EndEncounter { ended_at_ms: now_ms() });
        state.encounter = Encounter::default();
        state.skills_store.clear();

        if state.event_manager.should_emit_events() {
            state.event_manager.emit_encounter_reset();
        }
    }

    pub async fn get_encounter(&self) -> Encounter {
        self.state.read().await.encounter.clone()
    }

    pub async fn get_skills_store(&self) -> SkillsStore {
        self.state.read().await.skills_store.clone()
    }

    pub async fn update_skills_store<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut SkillsStore),
    {
        let mut state = self.state.write().await;
        update_fn(&mut state.skills_store);
    }

    /// Add a recent name entry into the backend cache. This is safe to call
    /// from packet processing paths; it acquires the AppState write lock.
    pub async fn add_recent_name(&self, uid: i64, name: String) {
        let mut state = self.state.write().await;
        state.recent_names.add_name(uid, name);
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

    pub async fn get_skills_store_ref(&self) -> tokio::sync::RwLockReadGuard<'_, AppState> {
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
        let (encounter, should_emit) = {
            let state = self.state.read().await;
            (state.encounter.clone(), state.event_manager.should_emit_events())
        };

        if !should_emit {
            return;
        }

        // Generate all the data we need without holding the lock
        let header_info = crate::live::event_manager::generate_header_info(&encounter);
        let dps_players = crate::live::event_manager::generate_players_window_dps(&encounter);
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
                if let Some(skills_window) =
                    crate::live::event_manager::generate_skills_window_dps(&encounter, entity_uid)
                {
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
                    crate::live::event_manager::generate_skills_window_tanked(&encounter, entity_uid)
                {
                    tanked_skill_windows.push((entity_uid, skills_window));
                }
            }

            // Collect subscribed players for later emission
            subscribed_players.push(entity_uid);
        }

        // Now, acquire the write lock and update everything
        let mut state = self.state.write().await;

        // Emit encounter update
        if let Some(header_info) = header_info {
            state.event_manager.emit_encounter_update(
                header_info,
                encounter.is_encounter_paused, // Use the original encounter state
            );
        }

        // Emit DPS players update
        if !dps_players.player_rows.is_empty() {
            state.event_manager.emit_players_update(MetricType::Dps, dps_players);
        }

        // Emit heal players update
        if !heal_players.player_rows.is_empty() {
            state.event_manager.emit_players_update(MetricType::Heal, heal_players);
        }

        // Emit tanked players update
        if !tanked_players.player_rows.is_empty() {
            state.event_manager.emit_players_update(MetricType::Tanked, tanked_players);
        }

        // Update skills store for all players with damage/heal data
        for (entity_uid, skills_window) in dps_skill_windows {
            state.skills_store.update_dps_skills(entity_uid, skills_window);
        }

        for (entity_uid, skills_window) in heal_skill_windows {
            state.skills_store.update_heal_skills(entity_uid, skills_window);
        }

        for (entity_uid, skills_window) in tanked_skill_windows {
            state.skills_store.update_tanked_skills(entity_uid, skills_window);
        }

        // Emit skills updates only for subscribed players
        for entity_uid in subscribed_players {
            if state.skills_store.is_subscribed(entity_uid, "dps") {
                if let Some(skills_window) = state.skills_store.get_dps_skills(entity_uid) {
                    state.event_manager.emit_skills_update(
                        MetricType::Dps,
                        entity_uid,
                        skills_window.clone(),
                    );
                }
            }
            if state.skills_store.is_subscribed(entity_uid, "heal") {
                if let Some(skills_window) = state.skills_store.get_heal_skills(entity_uid) {
                    state.event_manager.emit_skills_update(
                        MetricType::Heal,
                        entity_uid,
                        skills_window.clone(),
                    );
                }
            }
            if state.skills_store.is_subscribed(entity_uid, "tanked") {
                if let Some(skills_window) = state.skills_store.get_tanked_skills(entity_uid) {
                    state.event_manager.emit_skills_update(
                        MetricType::Tanked,
                        entity_uid,
                        skills_window.clone(),
                    );
                }
            }
        }
    }
}