/// Attempt detection module for boss splitting feature.
///
/// Detects when to split encounters into separate attempts based on:
/// - Raid wipes (all party members dead)
/// - Boss HP rollback (boss HP returns to >=90% after being lower)
use crate::database::{enqueue, DbTask};
use crate::live::opcodes_models::{AttrType, Encounter, Entity};
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::info;

/// Configuration for attempt detection thresholds.
#[derive(Debug, Clone)]
pub struct AttemptConfig {
    /// Minimum HP percentage decrease before a rollback can trigger a split (default: 60%)
    pub min_hp_decrease_pct: f64,
    /// HP percentage threshold for rollback detection (default: 90%)
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
            min_hp_decrease_pct: 80.0,
            hp_rollback_threshold_pct: 95.0,
            split_cooldown_ms: 2000,
            enable_wipe_detection: true,
            enable_hp_rollback_detection: true,
        }
    }
}

/// Checks if a wipe has occurred (all party members dead).
///
/// A wipe is detected when all players in the party have died recently.
/// This uses the pending_player_deaths queue to check for recent deaths.
pub fn check_wipe_condition(encounter: &Encounter, config: &AttemptConfig) -> bool {
    if !config.enable_wipe_detection {
        return false;
    }

    // Need at least one party member to detect a wipe
    if encounter.party_member_uids.is_empty() {
        return false;
    }

    // Check if all party members have died recently
    let all_dead = encounter.party_member_uids.iter().all(|uid| {
        encounter.last_death_ms.get(uid).is_some()
    });

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

    let Some(lowest_hp) = encounter.lowest_boss_hp else {
        return false;
    };

    // Calculate lowest HP as percentage
    // lowest_hp is stored as absolute HP value
    // We need max HP to calculate percentage
    // For now, assume current_pct is already calculated by caller
    // Check if we dropped below min threshold and are now above rollback threshold
    let dropped_significantly = lowest_hp < (config.min_hp_decrease_pct * 100.0) as i64;
    let rolled_back = current_pct >= config.hp_rollback_threshold_pct;

    dropped_significantly && rolled_back
}

/// Initiates an attempt split.
///
/// This function:
/// - Ends the current attempt
/// - Increments attempt index
/// - Starts a new attempt
/// - Updates encounter state
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

    // Count deaths in current attempt
    let deaths_in_attempt = encounter
        .pending_player_deaths
        .iter()
        .filter(|(_, _, _, ts)| *ts as u128 >= encounter.time_fight_start_ms)
        .count() as i32;

    // End current attempt
    enqueue(DbTask::EndAttempt {
        attempt_index: current_attempt,
        ended_at_ms: timestamp_ms as i64,
        boss_hp_end: boss_hp,
        total_deaths: deaths_in_attempt,
    });

    // Increment attempt index
    encounter.current_attempt_index += 1;

    // Start new attempt
    enqueue(DbTask::BeginAttempt {
        attempt_index: encounter.current_attempt_index,
        started_at_ms: timestamp_ms as i64,
        reason: reason.to_string(),
        boss_hp_start: boss_hp,
    });

    // Reset tracking for new attempt
    encounter.boss_hp_at_attempt_start = boss_hp;
    encounter.lowest_boss_hp = boss_hp;
    encounter.last_attempt_split_ms = timestamp_ms;
    encounter.pending_player_deaths.clear();
    encounter.last_death_ms.clear();

    info!(
        "Attempt split: {} -> Attempt {} (reason: {})",
        current_attempt,
        encounter.current_attempt_index,
        reason
    );
}

/// Updates encounter state with current boss HP and checks for rollback.
pub fn update_boss_hp_tracking(encounter: &mut Encounter, current_hp: i64) {
    // Track lowest HP seen
    if encounter.lowest_boss_hp.is_none() || current_hp < encounter.lowest_boss_hp.unwrap() {
        encounter.lowest_boss_hp = Some(current_hp);
    }
}

/// Tracks party member UIDs for wipe detection.
pub fn track_party_member(encounter: &mut Encounter, uid: i64, entity_type: EEntityType, team_id: Option<i64>) {
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
        assert_eq!(config.min_hp_decrease_pct, 60.0);
        assert_eq!(config.hp_rollback_threshold_pct, 90.0);
        assert_eq!(config.split_cooldown_ms, 2000);
        assert!(config.enable_wipe_detection);
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

        assert!(!check_hp_rollback_condition(&encounter, Some(95.0), &config));
    }
}
