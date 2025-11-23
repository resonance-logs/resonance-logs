use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

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
        emit_if_changed(&self.app_handle, snapshot);
    }

    pub fn snapshot(&self) -> Option<DungeonLog> {
        snapshot(&self.shared_log)
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

/// Emits the provided snapshot if available.
pub fn emit_if_changed(app_handle: &AppHandle, snapshot: Option<DungeonLog>) {
    if let Some(log) = snapshot {
        if let Err(err) = app_handle.emit("log-update", log) {
            error!("Failed to emit log-update: {err}");
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

        match self.combat_state {
            CombatState::Idle => {
                if event.is_boss_target {
                    self.start_boss_segment(event);
                    (true, false, true) // changed, boss_died, new_boss_started
                } else {
                    (self.log_trash_event(event), false, false)
                }
            }
            CombatState::InCombat => self.append_to_active_segment(event),
        }
    }

    fn start_boss_segment(&mut self, event: DamageEvent) {
        self.close_active_trash(event.timestamp_ms);
        let mut segment = Segment::new(SegmentType::Boss, event.timestamp_ms, self.next_segment_id);
        self.next_segment_id += 1;
        segment.boss_entity_id = Some(event.target_id);
        segment.boss_monster_type_id = event.target_monster_type_id;
        segment.boss_name = event.target_name.clone();
        segment.append_event(event);
        self.segments.push(segment);
        self.active_segment_idx = Some(self.segments.len() - 1);
        self.combat_state = CombatState::InCombat;
    }

    fn log_trash_event(&mut self, event: DamageEvent) -> bool {
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
        if let Some(idx) = self.active_segment_idx {
            if let Some(segment) = self.segments.get_mut(idx) {
                let is_killing = event.is_killing_blow
                    && (segment.boss_entity_id == Some(event.target_id)
                        || segment.boss_monster_type_id == event.target_monster_type_id);

                segment.append_event(event);

                if is_killing {
                    let timestamp_ms = segment
                        .events
                        .last()
                        .map(|ev| ev.timestamp_ms)
                        .unwrap_or_else(timestamp_now_ms);
                    segment.close(timestamp_ms);
                    self.active_segment_idx = None;
                    self.combat_state = CombatState::Idle;
                    return (true, true, false); // changed, boss_died, new_boss_started
                }
                (true, false, false)
            } else {
                (false, false, false)
            }
        } else {
            // No active boss segment, treat as trash
            self.combat_state = CombatState::Idle;
            (self.log_trash_event(event), false, false)
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

        if let Some(idx) = self.active_segment_idx.take() {
            if let Some(segment) = self.segments.get_mut(idx) {
                segment.close(timestamp_now_ms());
            }
        }
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
pub fn persist_segments(handle: &SharedDungeonLog) {
    use crate::database::{DbTask, enqueue};

    // Lock the log to mutate persistence state
    let mut log = match lock_log(handle) {
        Some(guard) => guard,
        None => return,
    };

    for segment in log.segments.iter_mut() {
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
) -> DamageEvent {
    let is_boss_target = target_monster_type_id
        .map(|id| GLOBAL_BOSS_LIST.contains(&id))
        .unwrap_or(false);
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
