use crate::live::opcodes_models::Encounter;
use crate::live::player_names::PlayerNames;
use crate::live::event_manager::{EventManager, MetricType};
use blueprotobuf_lib::blueprotobuf;
use log::{info, warn};
use std::collections::HashSet;
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
    pub skill_subscriptions: HashSet<(i64, String)>,
    pub app_handle: AppHandle,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            encounter: Encounter::default(),
            event_manager: EventManager::new(),
            skill_subscriptions: HashSet::new(),
            app_handle,
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
                self.process_sync_container_dirty_data(&mut state, data).await;
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
        enqueue(DbTask::EndEncounter { ended_at_ms: now_ms() });
        on_server_change(&mut state.encounter);

        // Emit encounter reset event
        if state.event_manager.should_emit_events() {
            state.event_manager.emit_encounter_reset();
        }

        // Clear skill subscriptions
        state.skill_subscriptions.clear();
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
        state.encounter.reset_combat_state();
        state.skill_subscriptions.clear();

        if state.event_manager.should_emit_events() {
            state.event_manager.emit_encounter_reset();
        }
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

        // Emit skills updates only for subscribed players using precomputed windows
        for (entity_uid, skills_window) in &dps_skill_windows {
            if state.skill_subscriptions.contains(&(*entity_uid, "dps".to_string())) {
                state.event_manager.emit_skills_update(
                    MetricType::Dps,
                    *entity_uid,
                    skills_window.clone(),
                );
            }
        }
        for (entity_uid, skills_window) in &heal_skill_windows {
            if state.skill_subscriptions.contains(&(*entity_uid, "heal".to_string())) {
                state.event_manager.emit_skills_update(
                    MetricType::Heal,
                    *entity_uid,
                    skills_window.clone(),
                );
            }
        }
        for (entity_uid, skills_window) in &tanked_skill_windows {
            if state.skill_subscriptions.contains(&(*entity_uid, "tanked".to_string())) {
                state.event_manager.emit_skills_update(
                    MetricType::Tanked,
                    *entity_uid,
                    skills_window.clone(),
                );
            }
        }
    }
}