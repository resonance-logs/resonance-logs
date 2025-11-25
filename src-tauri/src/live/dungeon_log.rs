use log::{trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

/// Shared handle that can be stored inside Tauri state.
pub type SharedDungeonLog = Arc<Mutex<DungeonLog>>;

/// Global timeout for ending a segment when no events were seen.
pub const SEGMENT_TIMEOUT: Duration = Duration::from_secs(15);

/// Monster IDs that are considered bosses.
pub static GLOBAL_BOSS_LIST: LazyLock<HashSet<i64>> = LazyLock::new(|| {
    let data = include_str!("../../meter-data/MonsterNameBoss.json");
    serde_json::from_str::<HashMap<String, String>>(data)
        .map(|map| {
            map.keys()
                .filter_map(|key| key.parse::<i64>().ok())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default()
});

/// Runtime helper that bundles the shared log handle with an app handle for emissions.
#[derive(Clone)]
pub struct DungeonLogRuntime {
    pub shared_log: SharedDungeonLog,
    pub app_handle: AppHandle,
}

impl DungeonLogRuntime {
    pub fn new(shared_log: SharedDungeonLog, app_handle: AppHandle) -> Self {
        Self {
            shared_log,
            app_handle,
        }
    }

    pub fn process_damage_event(&self, event: DamageEvent) -> (bool, bool) {
        let (snapshot, boss_died, new_boss_started) = process_damage_event(&self.shared_log, event);
        emit_if_changed(&self.app_handle, snapshot);
        (boss_died, new_boss_started)
    }

    pub fn reset_for_scene(&self, scene_id: Option<i32>, scene_name: Option<String>) {
        let snapshot = reset_for_scene(&self.shared_log, scene_id, scene_name);
        emit_if_changed(&self.app_handle, snapshot);
    }

    pub fn check_for_timeout(&self, now: Instant) {
        let snapshot = check_for_timeout(&self.shared_log, now, SEGMENT_TIMEOUT);
        if snapshot.is_some() {
            persist_segments(&self.shared_log, false);
        }
        emit_if_changed(&self.app_handle, snapshot);
    }

    pub fn snapshot(&self) -> Option<DungeonLog> {
        snapshot(&self.shared_log)
    }
}

/// Cached entity info for boss detection when attributes arrive late.
#[derive(Debug, Clone, Default)]
struct EntityCache {
    /// Maps entity_id -> (monster_type_id, boss_name)
    entities: HashMap<i64, (Option<i64>, Option<String>)>,
}

impl EntityCache {
    /// Update cache with entity info from a damage event.
    fn update(&mut self, event: &DamageEvent) {
        let entry = self.entities.entry(event.target_id).or_default();
        if event.target_monster_type_id.is_some() {
            entry.0 = event.target_monster_type_id;
        }
        if event.target_name.is_some() {
            entry.1 = event.target_name.clone();
        }
    }

    /// Check if an entity is known to be a boss from cached info.
    fn is_known_boss(&self, entity_id: i64) -> bool {
        self.entities
            .get(&entity_id)
            .and_then(|(monster_type_id, _)| *monster_type_id)
            .map(|id| GLOBAL_BOSS_LIST.contains(&id))
            .unwrap_or(false)
    }

    /// Get cached monster type id for an entity.
    fn get_monster_type_id(&self, entity_id: i64) -> Option<i64> {
        self.entities.get(&entity_id).and_then(|(id, _)| *id)
    }

    /// Clear the cache (e.g., on scene change).
    fn clear(&mut self) {
        self.entities.clear();
    }
}

/// Master container for dungeon segments within a scene.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DungeonLog {
    pub scene_id: Option<i32>,
    pub scene_name: Option<String>,
    pub combat_state: CombatState,
    pub segments: Vec<Segment>,
    #[serde(skip)]
    #[specta(skip)]
    active_segment_idx: Option<usize>,
    #[serde(skip)]
    #[specta(skip)]
    active_trash_idx: Option<usize>,
    #[serde(skip)]
    #[specta(skip)]
    last_event_at: Option<Instant>,
    #[serde(skip)]
    #[specta(skip)]
    next_segment_id: u64,
    /// Cache of entity info for late-arriving attributes.
    #[serde(skip)]
    #[specta(skip)]
    entity_cache: EntityCache,
}

impl Default for DungeonLog {
    fn default() -> Self {
        Self {
            scene_id: None,
            scene_name: None,
            combat_state: CombatState::Idle,
            segments: Vec::new(),
            active_segment_idx: None,
            active_trash_idx: None,
            last_event_at: None,
            next_segment_id: 1,
            entity_cache: EntityCache::default(),
        }
    }
}

/// Represents an individual combat segment (boss or trash).
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub id: u64,
    pub segment_type: SegmentType,
    pub boss_entity_id: Option<i64>,
    pub boss_monster_type_id: Option<i64>,
    pub boss_name: Option<String>,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub total_damage: i64,
    pub hit_count: u64,
    pub events: Vec<DamageEvent>,
    #[serde(skip)]
    #[specta(skip)]
    pub persisted: bool,
}

impl Segment {
    fn new(segment_type: SegmentType, timestamp_ms: i64, id: u64) -> Self {
        Self {
            id,
            segment_type,
            boss_entity_id: None,
            boss_monster_type_id: None,
            boss_name: None,
            started_at_ms: timestamp_ms,
            ended_at_ms: None,
            total_damage: 0,
            hit_count: 0,
            events: Vec::new(),
            persisted: false,
        }
    }

    fn append_event(&mut self, event: DamageEvent) {
        self.total_damage = self.total_damage.saturating_add(event.amount.max(0));
        self.hit_count = self.hit_count.saturating_add(1);
        self.events.push(event);
    }

    fn matches_boss_target(&mut self, event: &DamageEvent) -> bool {
        if self.segment_type != SegmentType::Boss {
            return false;
        }

        let entity_match = self.boss_entity_id == Some(event.target_id);
        let monster_match = match (self.boss_monster_type_id, event.target_monster_type_id) {
            (Some(existing), Some(incoming)) => existing == incoming,
            _ => false,
        };
        let name_match = self
            .boss_name
            .as_ref()
            .zip(event.target_name.as_ref())
            .map(|(a, b)| a.eq_ignore_ascii_case(b))
            .unwrap_or(false);

        if !entity_match && (monster_match || name_match) {
            self.boss_entity_id = Some(event.target_id);
        }

        if self.boss_monster_type_id.is_none()
            && event.target_monster_type_id.is_some()
            && (entity_match || name_match)
        {
            self.boss_monster_type_id = event.target_monster_type_id;
        }

        if self.boss_name.is_none() && event.target_name.is_some() {
            self.boss_name = event.target_name.clone();
        }

        entity_match || monster_match || name_match
    }

    fn close(&mut self, timestamp_ms: i64) {
        if self.ended_at_ms.is_none() {
            self.ended_at_ms = Some(timestamp_ms);
        }
    }
}

/// Discrete damage occurrence stored on a segment.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DamageEvent {
    pub timestamp_ms: i64,
    pub attacker_id: i64,
    pub target_id: i64,
    pub target_name: Option<String>,
    pub target_monster_type_id: Option<i64>,
    pub amount: i64,
    pub is_boss_target: bool,
    pub is_killing_blow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum SegmentType {
    Boss,
    Trash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum CombatState {
    Idle,
    InCombat,
}

/// Creates a new shared dungeon log handle.
pub fn create_shared_log() -> SharedDungeonLog {
    Arc::new(Mutex::new(DungeonLog::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boss_event(timestamp: i64, target_id: i64, killing: bool) -> DamageEvent {
        DamageEvent {
            timestamp_ms: timestamp,
            attacker_id: 1,
            target_id,
            target_name: Some("Test Boss".into()),
            target_monster_type_id: Some(42),
            amount: 1000,
            is_boss_target: true,
            is_killing_blow: killing,
        }
    }

    #[test]
    fn boss_segment_created_and_closed() {
        let mut log = DungeonLog::default();
        let (changed, boss_died, new_boss) =
            log.apply_damage_event(boss_event(100, 10, false), Instant::now());
        assert!(changed);
        assert!(new_boss);
        assert!(!boss_died);
        assert_eq!(log.segments.len(), 1);
        assert_eq!(log.combat_state, CombatState::InCombat);

        let (changed, boss_died, new_boss) =
            log.apply_damage_event(boss_event(200, 10, true), Instant::now());
        assert!(changed);
        assert!(boss_died);
        assert!(!new_boss);
        assert_eq!(log.segments.len(), 1);
        // Segment should remain open to support multi-entity bosses
        assert!(log.segments[0].ended_at_ms.is_none());
        assert_eq!(log.combat_state, CombatState::InCombat);
    }

    #[test]
    fn late_arriving_boss_detection_via_entity_cache() {
        // Test scenario: boss entity_id is damaged before monster_type_id is known
        // Later, we get the monster_type_id - should be recognized as same boss
        let mut log = DungeonLog::default();
        let entity_id = 42;

        // First event: entity is unknown (no monster_type_id), but later identified as boss
        // via is_boss_target hint (e.g., from EliteStatus attribute)
        let first_event = DamageEvent {
            timestamp_ms: 100,
            attacker_id: 1,
            target_id: entity_id,
            target_name: Some("Mystery Boss".into()),
            target_monster_type_id: None, // Unknown at first
            amount: 1000,
            is_boss_target: true, // Identified as boss via other means
            is_killing_blow: false,
        };

        let (changed, _, new_boss) = log.apply_damage_event(first_event, Instant::now());
        assert!(changed, "First boss event should create a segment");
        assert!(new_boss, "Should be flagged as new boss");
        assert_eq!(log.segments.len(), 1);
        assert_eq!(log.segments[0].segment_type, SegmentType::Boss);
        assert!(
            log.segments[0].boss_monster_type_id.is_none(),
            "Monster type should be unknown initially"
        );

        // Second event: same entity, now we have the monster_type_id
        // Boss monster type 10010 = Tempest Ogre (known boss)
        let second_event = DamageEvent {
            timestamp_ms: 200,
            attacker_id: 1,
            target_id: entity_id,
            target_name: Some("Tempest Ogre".into()),
            target_monster_type_id: Some(10010), // Now known
            amount: 1500,
            is_boss_target: true,
            is_killing_blow: false,
        };

        let (changed, _, new_boss) = log.apply_damage_event(second_event, Instant::now());
        assert!(changed, "Should update segment");
        assert!(!new_boss, "Should not be flagged as new boss - same entity");
        assert_eq!(log.segments.len(), 1, "Should still be same segment");

        // Check that segment was updated with monster_type_id
        assert_eq!(
            log.segments[0].boss_monster_type_id,
            Some(10010),
            "Boss monster type should be backfilled"
        );
    }

    #[test]
    fn entity_cache_helps_identify_boss_on_second_encounter() {
        // Test: entity is seen once with full info, then later seen without monster_type_id
        // The cache should help identify it as a boss
        let mut log = DungeonLog::default();
        let entity_id = 99;

        // First encounter with full info - Tempest Ogre (boss monster 10010)
        let first_event = DamageEvent {
            timestamp_ms: 100,
            attacker_id: 1,
            target_id: entity_id,
            target_name: Some("Tempest Ogre".into()),
            target_monster_type_id: Some(10010),
            amount: 1000,
            is_boss_target: true,
            is_killing_blow: true, // Boss dies
        };

        log.apply_damage_event(first_event, Instant::now());
        assert_eq!(log.segments.len(), 1);

        // Simulate timeout to close the segment
        log.handle_timeout(
            Instant::now() + std::time::Duration::from_secs(20),
            SEGMENT_TIMEOUT,
        );
        assert_eq!(log.combat_state, CombatState::Idle);

        // Second encounter - same entity, but this time monster_type_id is missing in packet
        let second_event = DamageEvent {
            timestamp_ms: 300,
            attacker_id: 1,
            target_id: entity_id,
            target_name: Some("Tempest Ogre".into()),
            target_monster_type_id: None, // Missing this time
            amount: 2000,
            is_boss_target: false, // Not flagged directly
            is_killing_blow: false,
        };

        let (changed, _, new_boss) = log.apply_damage_event(second_event, Instant::now());
        assert!(changed, "Should create new segment");
        // The cache should identify this as a boss from previous encounter
        assert!(
            log.entity_cache.is_known_boss(entity_id),
            "Entity should be cached as boss"
        );
        // Check if segment was created as boss (via entity cache lookup)
        assert!(
            log.segments
                .iter()
                .any(|s| s.boss_entity_id == Some(entity_id)),
            "Should match existing boss segment by entity_id"
        );
    }
}

/// Emits the provided snapshot if available.
/// Uses safe emission to handle WebView2 state errors gracefully.
pub fn emit_if_changed(app_handle: &AppHandle, snapshot: Option<DungeonLog>) {
    if let Some(log) = snapshot {
        // Check if windows are available before emitting
        let live_window = app_handle.get_webview_window(crate::WINDOW_LIVE_LABEL);
        let main_window = app_handle.get_webview_window(crate::WINDOW_MAIN_LABEL);

        if live_window.is_none() && main_window.is_none() {
            trace!("Skipping log-update emit: no windows available");
            return;
        }

        if let Err(err) = app_handle.emit("log-update", log) {
            // Check if this is a WebView2 state error (0x8007139F)
            let error_str = format!("{:?}", err);
            if error_str.contains("0x8007139F") || error_str.contains("not in the correct state") {
                // This is expected when windows are minimized/hidden - don't spam logs
                trace!("WebView2 not ready for log-update (window may be minimized/hidden)");
            } else {
                warn!("Failed to emit log-update: {}", err);
            }
        }
    }
}

/// Processes a damage event and returns (snapshot if mutated, boss_died, new_boss_started).
pub fn process_damage_event(
    handle: &SharedDungeonLog,
    event: DamageEvent,
) -> (Option<DungeonLog>, bool, bool) {
    let now = Instant::now();
    let mut log = match lock_log(handle) {
        Some(guard) => guard,
        None => return (None, false, false),
    };
    let (changed, boss_died, new_boss_started) = log.apply_damage_event(event, now);
    if changed {
        (Some(log.clone()), boss_died, new_boss_started)
    } else {
        (None, boss_died, new_boss_started)
    }
}

/// Resets the log when a new scene is detected and returns a snapshot if it changed.
pub fn reset_for_scene(
    handle: &SharedDungeonLog,
    scene_id: Option<i32>,
    scene_name: Option<String>,
) -> Option<DungeonLog> {
    let mut log = lock_log(handle)?;
    let changed = log.reset_if_scene_changed(scene_id, scene_name);
    if changed { Some(log.clone()) } else { None }
}

/// Clears the log completely.
pub fn clear(handle: &SharedDungeonLog) -> Option<DungeonLog> {
    let mut log = lock_log(handle)?;
    *log = DungeonLog::default();
    Some(log.clone())
}

/// Returns a snapshot of the log for the frontend.
pub fn snapshot(handle: &SharedDungeonLog) -> Option<DungeonLog> {
    lock_log(handle).map(|log| log.clone())
}

/// Checks for inactivity timeouts and closes an active segment if necessary.
pub fn check_for_timeout(
    handle: &SharedDungeonLog,
    now: Instant,
    timeout: Duration,
) -> Option<DungeonLog> {
    let mut log = lock_log(handle)?;
    let changed = log.handle_timeout(now, timeout);
    if changed { Some(log.clone()) } else { None }
}

fn lock_log(handle: &SharedDungeonLog) -> Option<MutexGuard<'_, DungeonLog>> {
    match handle.lock() {
        Ok(guard) => Some(guard),
        Err(poisoned) => {
            warn!("Dungeon log mutex poisoned, recovering state");
            Some(poisoned.into_inner())
        }
    }
}

impl DungeonLog {
    fn should_treat_as_trash(&self, event: &DamageEvent) -> bool {
        if event.amount <= 0 {
            return false;
        }

        // First check the event's monster type id
        if let Some(monster_type_id) = event.target_monster_type_id {
            return !GLOBAL_BOSS_LIST.contains(&monster_type_id);
        }

        // If no monster type in event, check our cache for this entity
        if let Some(cached_monster_type_id) = self.entity_cache.get_monster_type_id(event.target_id)
        {
            return !GLOBAL_BOSS_LIST.contains(&cached_monster_type_id);
        }

        // No info available - don't treat as trash (might be a boss we don't know about yet)
        false
    }

    fn is_boss_event(&self, event: &DamageEvent) -> bool {
        if event.is_boss_target {
            return true;
        }

        // Check using event's monster type id
        if let Some(monster_type_id) = event.target_monster_type_id {
            if GLOBAL_BOSS_LIST.contains(&monster_type_id) {
                return true;
            }
        }

        // Check using cached entity info (handles late-arriving attributes)
        if self.entity_cache.is_known_boss(event.target_id) {
            return true;
        }

        // Check if this entity matches any existing boss segment
        self.segments.iter().any(|segment| {
            segment.segment_type == SegmentType::Boss
                && (segment.boss_entity_id == Some(event.target_id)
                    || segment.boss_monster_type_id == event.target_monster_type_id)
        })
    }

    fn reset_if_scene_changed(
        &mut self,
        scene_id: Option<i32>,
        scene_name: Option<String>,
    ) -> bool {
        let scene_changed = match (self.scene_id, scene_id) {
            (Some(current), Some(new_id)) => current != new_id,
            (None, Some(_)) => true,
            (Some(_), None) => true,
            (None, None) => self
                .scene_name
                .as_ref()
                .zip(scene_name.as_ref())
                .map(|(a, b)| a != b)
                .unwrap_or(false),
        };

        if scene_changed {
            // Clear entity cache on scene change
            self.entity_cache.clear();
            *self = DungeonLog {
                scene_id,
                scene_name,
                ..DungeonLog::default()
            };
            true
        } else {
            false
        }
    }

    fn apply_damage_event(&mut self, event: DamageEvent, now: Instant) -> (bool, bool, bool) {
        self.last_event_at = Some(now);

        // Update entity cache with any new info from this event
        self.entity_cache.update(&event);

        match self.combat_state {
            CombatState::Idle => {
                if self.is_boss_event(&event) {
                    // Check if we have an existing open boss segment for this same boss
                    let existing_boss_segment_idx = self
                        .segments
                        .iter()
                        .enumerate()
                        .rev()
                        .find(|(_, s)| {
                            s.segment_type == SegmentType::Boss
                                && s.ended_at_ms.is_none()
                                && s.boss_entity_id == Some(event.target_id)
                        })
                        .map(|(idx, _)| idx);

                    if let Some(idx) = existing_boss_segment_idx {
                        // Resume existing segment for same boss
                        if let Some(segment) = self.segments.get_mut(idx) {
                            segment.append_event(event);
                            self.active_segment_idx = Some(idx);
                            self.combat_state = CombatState::InCombat;
                            (true, false, false) // changed, boss_died, new_boss_started
                        } else {
                            (false, false, false)
                        }
                    } else {
                        // Create new segment for new/different boss
                        let new_boss = self.start_boss_segment(event);
                        (true, false, new_boss) // changed, boss_died, new_boss_started
                    }
                } else {
                    (self.log_trash_event(event), false, false)
                }
            }
            CombatState::InCombat => self.append_to_active_segment(event),
        }
    }

    fn start_boss_segment(&mut self, event: DamageEvent) -> bool {
        self.close_active_trash(event.timestamp_ms);

        // Check if this is the same boss as the last boss segment
        // If it is, we don't want to trigger a "new boss" reset in the live meter
        let is_new_boss = if let Some(last_boss) = self
            .segments
            .iter()
            .rev()
            .find(|s| s.segment_type == SegmentType::Boss)
        {
            last_boss.boss_entity_id != Some(event.target_id)
        } else {
            true
        };

        let mut segment = Segment::new(SegmentType::Boss, event.timestamp_ms, self.next_segment_id);
        self.next_segment_id += 1;
        segment.boss_entity_id = Some(event.target_id);
        // Use event's monster type id, or fall back to cached value
        segment.boss_monster_type_id = event
            .target_monster_type_id
            .or_else(|| self.entity_cache.get_monster_type_id(event.target_id));
        segment.boss_name = event.target_name.clone();
        segment.append_event(event);
        self.segments.push(segment);
        self.active_segment_idx = Some(self.segments.len() - 1);
        self.combat_state = CombatState::InCombat;

        is_new_boss
    }

    fn log_trash_event(&mut self, event: DamageEvent) -> bool {
        if !self.should_treat_as_trash(&event) {
            return false;
        }

        let idx = match self.active_trash_idx {
            Some(idx) => {
                if self
                    .segments
                    .get(idx)
                    .map(|segment| segment.ended_at_ms.is_none())
                    .unwrap_or(false)
                {
                    idx
                } else {
                    self.create_trash_segment(event.timestamp_ms)
                }
            }
            None => self.create_trash_segment(event.timestamp_ms),
        };

        if let Some(segment) = self.segments.get_mut(idx) {
            segment.append_event(event);
            true
        } else {
            false
        }
    }

    fn create_trash_segment(&mut self, timestamp_ms: i64) -> usize {
        let segment = Segment::new(SegmentType::Trash, timestamp_ms, self.next_segment_id);
        self.next_segment_id += 1;
        self.segments.push(segment);
        let idx = self.segments.len() - 1;
        self.active_trash_idx = Some(idx);
        idx
    }

    fn append_to_active_segment(&mut self, event: DamageEvent) -> (bool, bool, bool) {
        let is_boss_event = self.is_boss_event(&event);

        if let Some(idx) = self.active_segment_idx {
            if let Some(segment) = self.segments.get_mut(idx) {
                // Check if this event belongs to the active segment
                let belongs_to_segment = match segment.segment_type {
                    SegmentType::Boss => segment.matches_boss_target(&event),
                    SegmentType::Trash => !is_boss_event,
                };

                if !belongs_to_segment {
                    // Event doesn't belong to active segment
                    if is_boss_event {
                        // Different boss - close active segment and create new one
                        segment.close(event.timestamp_ms);
                        self.active_segment_idx = None;
                        let new_boss = self.start_boss_segment(event);
                        return (true, false, new_boss);
                    } else if segment.segment_type == SegmentType::Boss {
                        // Non-boss damage during a boss fight (cleave on adds, etc.).
                        // Keep the boss segment active but log the trash hit separately.
                        let changed = self.log_trash_event(event);
                        return (changed, false, false);
                    } else {
                        // Trash during boss fight - close boss segment, go to Idle, log as trash
                        segment.close(event.timestamp_ms);
                        self.active_segment_idx = None;
                        self.combat_state = CombatState::Idle;
                        return (self.log_trash_event(event), false, false);
                    }
                }

                // Event belongs to segment - append it
                let is_killing = event.is_killing_blow
                    && (segment.boss_entity_id == Some(event.target_id)
                        || segment.boss_monster_type_id == event.target_monster_type_id);

                segment.append_event(event);

                // Keep the segment open even after a killing blow to support multi-entity bosses
                // Segment will close via timeout or when switching to a different boss
                if is_killing {
                    return (true, true, false); // changed, boss_died, new_boss_started
                }
                (true, false, false)
            } else {
                (false, false, false)
            }
        } else {
            // No active segment - check if this is a boss event
            if is_boss_event {
                // Boss event but no active segment - check for existing open segment or create new one
                let existing_boss_segment_idx = self
                    .segments
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, s)| {
                        s.segment_type == SegmentType::Boss
                            && s.ended_at_ms.is_none()
                            && s.boss_entity_id == Some(event.target_id)
                    })
                    .map(|(idx, _)| idx);

                if let Some(idx) = existing_boss_segment_idx {
                    // Resume existing segment
                    if let Some(segment) = self.segments.get_mut(idx) {
                        segment.append_event(event);
                        self.active_segment_idx = Some(idx);
                        // Stay in InCombat state
                        (true, false, false)
                    } else {
                        (false, false, false)
                    }
                } else {
                    // Create new boss segment
                    let new_boss = self.start_boss_segment(event);
                    (true, false, new_boss)
                }
            } else {
                // Not a boss event, treat as trash and go to Idle
                self.combat_state = CombatState::Idle;
                (self.log_trash_event(event), false, false)
            }
        }
    }

    fn close_active_trash(&mut self, timestamp_ms: i64) {
        if let Some(idx) = self.active_trash_idx.take() {
            if let Some(segment) = self.segments.get_mut(idx) {
                segment.close(timestamp_ms);
            }
        }
    }

    fn handle_timeout(&mut self, now: Instant, timeout: Duration) -> bool {
        if self.combat_state != CombatState::InCombat {
            return false;
        }

        let Some(last_event) = self.last_event_at else {
            return false;
        };

        if now.duration_since(last_event) < timeout {
            return false;
        }

        let mut keep_active_idx = None;
        if let Some(idx) = self.active_segment_idx {
            if let Some(segment) = self.segments.get_mut(idx) {
                if segment.segment_type == SegmentType::Boss {
                    keep_active_idx = Some(idx);
                } else {
                    segment.close(timestamp_now_ms());
                }
            }
        }

        self.active_segment_idx = keep_active_idx;
        self.combat_state = CombatState::Idle;
        self.last_event_at = Some(now);
        true
    }
}

fn timestamp_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

/// Persists all closed segments to the database.
pub fn persist_segments(handle: &SharedDungeonLog, force_close: bool) {
    use crate::database::{DbTask, enqueue};

    // Lock the log to mutate persistence state
    let mut log = match lock_log(handle) {
        Some(guard) => guard,
        None => return,
    };

    let now = timestamp_now_ms();

    for segment in log.segments.iter_mut() {
        if force_close && segment.ended_at_ms.is_none() {
            segment.close(now);
        }

        // Only persist closed segments that haven't been persisted yet
        if segment.ended_at_ms.is_none() || segment.persisted {
            continue;
        }

        let segment_type = match segment.segment_type {
            SegmentType::Boss => "boss",
            SegmentType::Trash => "trash",
        };

        enqueue(DbTask::InsertDungeonSegment {
            segment_type: segment_type.to_string(),
            boss_entity_id: segment.boss_entity_id,
            boss_monster_type_id: segment.boss_monster_type_id,
            boss_name: segment.boss_name.clone(),
            started_at_ms: segment.started_at_ms,
            ended_at_ms: segment.ended_at_ms,
            total_damage: segment.total_damage,
            hit_count: segment.hit_count as i64,
        });

        segment.persisted = true;
    }
}

/// Helper to construct a damage event from raw values.
pub fn build_damage_event(
    timestamp_ms: i64,
    attacker_id: i64,
    target_id: i64,
    target_name: Option<String>,
    target_monster_type_id: Option<i64>,
    amount: i64,
    is_killing_blow: bool,
    is_boss_target_hint: bool,
) -> DamageEvent {
    let is_boss_target = if is_boss_target_hint {
        true
    } else {
        target_monster_type_id
            .map(|id| GLOBAL_BOSS_LIST.contains(&id))
            .unwrap_or(false)
    };
    let sanitized_amount = amount.max(0);

    DamageEvent {
        timestamp_ms,
        attacker_id,
        target_id,
        target_name,
        target_monster_type_id,
        amount: sanitized_amount,
        is_boss_target,
        is_killing_blow,
    }
}
