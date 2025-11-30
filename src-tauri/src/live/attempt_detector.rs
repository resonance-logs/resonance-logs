/// Attempt detection module for boss splitting feature.
///
/// Detects when to split encounters into separate attempts based on:
/// - Optionally: raid wipes (all party members dead) — disabled by default
/// - Boss HP rollback (boss HP returns to >95% after being lower)
use crate::database::{DbTask, enqueue};
use crate::live::opcodes_models::{AttrType, Encounter, Entity};
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::info;

/// Configuration for attempt detection thresholds.
#[derive(Debug, Clone)]
pub struct AttemptConfig {
    /// Minimum HP percentage decrease before a rollback can trigger a split (default: 60%)
    pub min_hp_decrease_pct: f64,
    /// HP percentage threshold for rollback detection (default: 95%)
    pub hp_rollback_threshold_pct: f64,
    /// Cooldown period in milliseconds to avoid rapid splits (default: 2000ms)
    pub split_cooldown_ms: u128,
    /// Enable automatic splitting on wipe detection
    pub enable_wipe_detection: bool,
    /// Enable automatic splitting on HP rollback
    pub enable_hp_rollback_detection: bool,
}

impl Default for AttemptConfig {
    fn default() -> Self {
        Self {
            // Use conservative defaults: consider a rollback when boss dropped below 60% then later
            // returned to >95% — for wipe/reset detection we consider any return above
            // 95% from a previously lower value as a reset.
            min_hp_decrease_pct: 95.0,
            hp_rollback_threshold_pct: 95.0,
            split_cooldown_ms: 2000,
            // We disable death-based wipe detection by default since player death tracking
            // is unreliable; prefer HP rollback detection.
            enable_wipe_detection: true,
            enable_hp_rollback_detection: true,
        }
    }
}

/// Checks if a wipe has occurred (all party members dead).
///
/// A wipe is detected when all players in the party have died recently.
/// This uses the death DB dedupe map to check for recent death events when
/// wipe detection is enabled. Note: wipe detection is disabled by default.
pub fn check_wipe_condition(encounter: &Encounter, config: &AttemptConfig) -> bool {
    if !config.enable_wipe_detection {
        return false;
    }

    // Need at least one party member to detect a wipe
    if encounter.party_member_uids.is_empty() {
        return false;
    }

    // Check if all party members have recorded death DB timestamps recently
    let all_dead = encounter
        .party_member_uids
        .iter()
        .all(|uid| encounter.last_death_db_ms.get(uid).is_some());

    all_dead && encounter.party_member_uids.len() >= 1
}

/// Checks if boss HP has rolled back significantly.
///
/// Rollback is detected when:
/// - Boss HP dropped below min_hp_decrease_pct (e.g., < 60%)
/// - Boss HP is now >= hp_rollback_threshold_pct (e.g., >= 90%)
pub fn check_hp_rollback_condition(
    encounter: &Encounter,
    current_boss_hp_pct: Option<f64>,
    config: &AttemptConfig,
) -> bool {
    if !config.enable_hp_rollback_detection {
        return false;
    }

    let Some(current_pct) = current_boss_hp_pct else {
        return false;
    };

    // lowest_boss_hp is stored as a percentage in Encounter (0.0 - 100.0)
    let Some(lowest_pct) = encounter.lowest_boss_hp else {
        return false;
    };

    // Detect a rollback when the boss was ever below the configured minimum and has now
    // returned strictly above the rollback threshold (use > to avoid edge equality cases).
    let dropped_below_threshold = lowest_pct < config.min_hp_decrease_pct;
    let rolled_back = current_pct > config.hp_rollback_threshold_pct;

    dropped_below_threshold && rolled_back
}

/// Initiates an attempt split.
///
/// This function:
/// - Ends the current attempt
/// - Increments attempt index
/// - Starts a new attempt
/// - Updates encounter state
/// - Handles phase outcomes for wipes
pub fn split_attempt(
    encounter: &mut Encounter,
    reason: &str,
    timestamp_ms: u128,
    boss_hp: Option<i64>,
) {
    // Check cooldown to avoid rapid splits
    if timestamp_ms - encounter.last_attempt_split_ms < 2000 {
        return;
    }

    let current_attempt = encounter.current_attempt_index;

    // Count revives in current attempt (we track revives instead of deaths for UI).
    let deaths_in_attempt = encounter
        .pending_player_revives
        .iter()
        .filter(|(_, _, _, ts)| *ts as u128 >= encounter.time_fight_start_ms)
        .count() as i32;

    // Resolve boss HP to store: prefer the provided value, then the current boss entity HP,
    // then fall back to the attempt-start HP if available.
    let resolved_boss_hp = boss_hp
        .or_else(|| {
            encounter
                .entity_uid_to_entity
                .values()
                .find(|e| e.is_boss())
                .and_then(|e| e.hp())
        })
        .or(encounter.boss_hp_at_attempt_start);

    // End current attempt
    enqueue(DbTask::EndAttempt {
        attempt_index: current_attempt,
        ended_at_ms: timestamp_ms as i64,
        boss_hp_end: resolved_boss_hp,
        total_deaths: deaths_in_attempt,
    });

    // Increment attempt index
    encounter.current_attempt_index += 1;

    // Start new attempt
    enqueue(DbTask::BeginAttempt {
        attempt_index: encounter.current_attempt_index,
        started_at_ms: timestamp_ms as i64,
        reason: reason.to_string(),
        boss_hp_start: resolved_boss_hp,
    });

    // Reset tracking for new attempt
    encounter.boss_hp_at_attempt_start = resolved_boss_hp;
    // Initialize lowest seen HP percentage to the starting boss HP (if available)
    if let Some(bhp) = resolved_boss_hp {
        // Try to compute percentage using the boss max HP if present
        let pct = encounter
            .entity_uid_to_entity
            .values()
            .find(|e| e.is_boss())
            .and_then(|e| e.max_hp().map(|max| (bhp as f64 / max as f64) * 100.0));
        encounter.lowest_boss_hp = pct;
    } else {
        encounter.lowest_boss_hp = None;
    }
    encounter.last_attempt_split_ms = timestamp_ms;
    encounter.pending_player_revives.clear();
    encounter.last_revive_ms.clear();

    info!(
        "Attempt split: {} -> Attempt {} (reason: {})",
        current_attempt, encounter.current_attempt_index, reason
    );
}

/// Updates encounter state with current boss HP and checks for rollback.
pub fn update_boss_hp_tracking(encounter: &mut Encounter, current_hp: i64) {
    // Track lowest HP seen as a percentage (0.0 - 100.0). We need the boss max HP
    // to convert absolute HP to percent.
    if let Some(max_hp) = encounter
        .entity_uid_to_entity
        .values()
        .find(|e| e.is_boss())
        .and_then(|e| e.max_hp())
    {
        if max_hp > 0 {
            let current_pct = (current_hp as f64 / max_hp as f64) * 100.0;
            if encounter.lowest_boss_hp.is_none() || current_pct < encounter.lowest_boss_hp.unwrap()
            {
                encounter.lowest_boss_hp = Some(current_pct);
            }
        }
    }
}

/// Tracks party member UIDs for wipe detection.
pub fn track_party_member(
    encounter: &mut Encounter,
    uid: i64,
    entity_type: EEntityType,
    team_id: Option<i64>,
) {
    if entity_type == EEntityType::EntChar {
        // Add player to party tracking
        // Use team_id to determine if they're in the same party
        if let Some(team) = team_id {
            if let Some(local_team) = encounter
                .entity_uid_to_entity
                .get(&encounter.local_player_uid)
                .and_then(|e| e.team_id())
            {
                if team == local_team {
                    encounter.party_member_uids.insert(uid);
                }
            }
        }
    }
}

/// Gets the current boss HP percentage if a boss is present.
pub fn get_boss_hp_percentage(encounter: &Encounter) -> Option<f64> {
    // Find first boss entity
    for entity in encounter.entity_uid_to_entity.values() {
        if entity.is_boss() {
            if let (Some(current_hp), Some(max_hp)) = (entity.hp(), entity.max_hp()) {
                if max_hp > 0 {
                    return Some((current_hp as f64 / max_hp as f64) * 100.0);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AttemptConfig::default();
        assert_eq!(config.min_hp_decrease_pct, 95.0);
        assert_eq!(config.hp_rollback_threshold_pct, 95.0);
        assert_eq!(config.split_cooldown_ms, 2000);
        assert!(!config.enable_wipe_detection);
        assert!(config.enable_hp_rollback_detection);
    }

    #[test]
    fn test_wipe_detection_disabled() {
        let encounter = Encounter::default();
        let mut config = AttemptConfig::default();
        config.enable_wipe_detection = false;

        assert!(!check_wipe_condition(&encounter, &config));
    }

    #[test]
    fn test_hp_rollback_detection_disabled() {
        let encounter = Encounter::default();
        let mut config = AttemptConfig::default();
        config.enable_hp_rollback_detection = false;

        assert!(!check_hp_rollback_condition(
            &encounter,
            Some(95.0),
            &config
        ));
    }
}
