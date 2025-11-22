/// Phase detection module for encounter mob/boss splitting.
///
/// Detects when to transition between mob and boss phases based on:
/// - Boss entity detection in the encounter
/// - Wipe scenarios during different phases
/// - Combat activity timeout for downtime detection
use crate::database::{enqueue, DbTask};
use crate::live::opcodes_models::{Encounter, PhaseType};
use log::info;

/// Combat timeout threshold in milliseconds (15 seconds)
/// If no combat activity occurs for this duration, the current phase ends
pub const COMBAT_TIMEOUT_MS: u128 = 15000;

/// Phase context metadata for tracking phase state
#[derive(Debug, Clone)]
pub struct PhaseContext {
    pub boss_ids: Vec<i64>,
    pub is_combat_active: bool,
}

/// Checks for phase timeout due to combat inactivity.
///
/// If no combat activity has occurred for COMBAT_TIMEOUT_MS, the current phase should end.
pub fn check_phase_timeout(encounter: &Encounter, current_time_ms: u128) -> bool {
    if encounter.current_phase.is_none() {
        return false;
    }

    let time_since_last_combat = current_time_ms.saturating_sub(encounter.time_last_combat_packet_ms);
    time_since_last_combat >= COMBAT_TIMEOUT_MS
}

/// Helper function to determine if a mob phase should be split.
///
/// Mob phases split when there's a significant gap in combat activity (timeout).
pub fn should_split_mob_phase(encounter: &Encounter, current_time_ms: u128) -> bool {
    if encounter.current_phase != Some(PhaseType::Mob) {
        return false;
    }

    check_phase_timeout(encounter, current_time_ms)
}

/// Checks if the encounter should transition to boss phase.
///
/// This is triggered when a boss entity is first detected in the encounter.
pub fn check_boss_phase_transition(encounter: &Encounter) -> bool {
    // Check if any boss entity exists that's not already tracked
    encounter
        .entity_uid_to_entity
        .iter()
        .any(|(uid, entity)| {
            entity.is_boss()
                && !encounter.active_boss_ids.contains(uid)
                && !encounter.defeated_boss_ids.contains(uid)
        })
}

/// Begins a new boss phase for a specific boss entity.
pub fn begin_boss_phase(encounter: &mut Encounter, boss_id: i64, timestamp_ms: u128) {
    info!(
        "Beginning boss phase for boss ID {} at timestamp {}",
        boss_id, timestamp_ms
    );

    encounter.clear_intermission_pause();
    encounter.current_phase = Some(PhaseType::Boss);
    encounter.phase_start_ms = timestamp_ms;
    encounter.active_boss_ids.insert(boss_id);
    encounter.boss_detected = true;

    enqueue(DbTask::BeginPhase {
        phase_type: "boss".to_string(),
        start_time_ms: timestamp_ms as i64,
    });
}

/// Begins a new mob phase.
pub fn begin_mob_phase(encounter: &mut Encounter, timestamp_ms: u128) {
    info!("Beginning mob phase at timestamp {}", timestamp_ms);

    encounter.clear_intermission_pause();
    encounter.current_phase = Some(PhaseType::Mob);
    encounter.phase_start_ms = timestamp_ms;

    enqueue(DbTask::BeginPhase {
        phase_type: "mob".to_string(),
        start_time_ms: timestamp_ms as i64,
    });
}

/// Begins a new phase for the current encounter (legacy function for compatibility).
pub fn begin_phase(encounter: &mut Encounter, phase_type: PhaseType, timestamp_ms: u128) {
    match phase_type {
        PhaseType::Mob => begin_mob_phase(encounter, timestamp_ms),
        PhaseType::Boss => {
            // For legacy compatibility, just begin a boss phase without specific ID
            info!("Beginning boss phase (legacy) at timestamp {}", timestamp_ms);
            encounter.clear_intermission_pause();
            encounter.current_phase = Some(PhaseType::Boss);
            encounter.phase_start_ms = timestamp_ms;
            encounter.boss_detected = true;

            enqueue(DbTask::BeginPhase {
                phase_type: "boss".to_string(),
                start_time_ms: timestamp_ms as i64,
            });
        }
    }
}

/// Ends the current phase for the encounter.
pub fn end_phase(encounter: &mut Encounter, outcome: &str, timestamp_ms: u128) {
    if let Some(phase_type) = encounter.current_phase {
        let phase_str = match phase_type {
            PhaseType::Mob => "mob",
            PhaseType::Boss => "boss",
        };

        // Outcome is constrained in the DB; map unsupported values to "unknown".
        let normalized_outcome = match outcome {
            "success" | "wipe" | "unknown" => outcome,
            _ => "unknown",
        };

        info!(
            "Ending {} phase with outcome '{}' at timestamp {}",
            phase_str, normalized_outcome, timestamp_ms
        );

        enqueue(DbTask::EndPhase {
            phase_type: phase_str.to_string(),
            end_time_ms: timestamp_ms as i64,
            outcome: normalized_outcome.to_string(),
        });

        encounter.current_phase = None;
        encounter.set_intermission_pause(true, Some(timestamp_ms));
    }
}

/// Handles boss death event.
pub fn handle_boss_death(encounter: &mut Encounter, boss_id: i64, timestamp_ms: u128) {
    info!(
        "Boss {} died at timestamp {}",
        boss_id, timestamp_ms
    );

    // Remove boss from active tracking
    encounter.active_boss_ids.remove(&boss_id);

    // If this was the last active boss, end the boss phase
    if encounter.active_boss_ids.is_empty() && encounter.current_phase == Some(PhaseType::Boss) {
        end_phase(encounter, "success", timestamp_ms);
    }
}

/// Handles the transition from mob to boss phase.
pub fn transition_to_boss_phase(encounter: &mut Encounter, boss_id: i64, timestamp_ms: u128) {
    // End mob phase with success
    if encounter.current_phase == Some(PhaseType::Mob) {
        end_phase(encounter, "success", timestamp_ms);
    }

    // Begin boss phase for the specific boss
    begin_boss_phase(encounter, boss_id, timestamp_ms);
}

/// Calculates individual phase duration in seconds.
pub fn get_phase_duration(start_time_ms: i64, end_time_ms: Option<i64>) -> f64 {
    if let Some(end) = end_time_ms {
        let duration_ms = end.saturating_sub(start_time_ms);
        duration_ms as f64 / 1000.0
    } else {
        0.0
    }
}

/// Calculates total active combat time by summing all phase durations.
///
/// This excludes downtime (gaps between phases) from the calculation.
/// Returns the total active combat time in seconds.
pub fn calculate_active_combat_time(
    phases: &[(i64, Option<i64>)] // Vec of (start_time_ms, end_time_ms)
) -> f64 {
    phases
        .iter()
        .map(|(start, end)| get_phase_duration(*start, *end))
        .sum()
}

/// Handles wipe scenarios based on the current phase.
pub fn handle_wipe(encounter: &mut Encounter, timestamp_ms: u128) {
    if let Some(phase_type) = encounter.current_phase {
        match phase_type {
            PhaseType::Mob => {
                // Wipe during mob phase: end mob phase with wipe outcome, no boss phase
                end_phase(encounter, "wipe", timestamp_ms);
            }
            PhaseType::Boss => {
                // Wipe during boss phase: end boss phase with wipe outcome
                // Mob phase (if it existed) remains with its previous outcome
                end_phase(encounter, "wipe", timestamp_ms);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_transitions() {
        let mut encounter = Encounter::default();

        // Start with no phase
        assert_eq!(encounter.current_phase, None);
        assert!(!encounter.boss_detected);

        // Begin mob phase
        begin_phase(&mut encounter, PhaseType::Mob, 1000);
        assert_eq!(encounter.current_phase, Some(PhaseType::Mob));
        assert_eq!(encounter.phase_start_ms, 1000);

        // Transition to boss phase (using boss ID 100)
        transition_to_boss_phase(&mut encounter, 100, 2000);
        assert_eq!(encounter.current_phase, Some(PhaseType::Boss));
        assert!(encounter.boss_detected);
        assert_eq!(encounter.phase_start_ms, 2000);
        assert_eq!(encounter.active_boss_ids.len(), 1);
        assert!(encounter.active_boss_ids.contains(&100));

        // Handle wipe in boss phase
        handle_wipe(&mut encounter, 3000);
        assert_eq!(encounter.current_phase, None);
    }

    #[test]
    fn test_mob_phase_wipe() {
        let mut encounter = Encounter::default();

        // Begin mob phase
        begin_phase(&mut encounter, PhaseType::Mob, 1000);
        assert_eq!(encounter.current_phase, Some(PhaseType::Mob));

        // Wipe during mob phase - should end phase without transitioning to boss
        handle_wipe(&mut encounter, 2000);
        assert_eq!(encounter.current_phase, None);
        assert!(!encounter.boss_detected);
    }

    #[test]
    fn test_boss_phase_wipe() {
        let mut encounter = Encounter::default();

        // Begin and complete mob phase
        begin_phase(&mut encounter, PhaseType::Mob, 1000);
        transition_to_boss_phase(&mut encounter, 100, 2000);
        assert_eq!(encounter.current_phase, Some(PhaseType::Boss));
        assert!(encounter.boss_detected);
        assert!(encounter.active_boss_ids.contains(&100));

        // Wipe during boss phase
        handle_wipe(&mut encounter, 3000);
        assert_eq!(encounter.current_phase, None);
        assert!(encounter.boss_detected); // Boss was detected, even though we wiped
    }

    #[test]
    fn test_boss_detection_prevents_retransition() {
        let mut encounter = Encounter::default();

        // Begin mob phase
        begin_phase(&mut encounter, PhaseType::Mob, 1000);

        // Transition to boss phase (legacy function)
        begin_phase(&mut encounter, PhaseType::Boss, 2000);
        assert!(encounter.boss_detected);

        // Should not detect boss phase transition for already-tracked bosses
        // (needs actual boss entities to test properly)
    }

    #[test]
    fn test_multi_boss_tracking() {
        let mut encounter = Encounter::default();

        // Start with mob phase
        begin_mob_phase(&mut encounter, 1000);
        assert_eq!(encounter.current_phase, Some(PhaseType::Mob));

        // First boss detected
        transition_to_boss_phase(&mut encounter, 100, 2000);
        assert_eq!(encounter.active_boss_ids.len(), 1);
        assert!(encounter.active_boss_ids.contains(&100));
        assert_eq!(encounter.current_phase, Some(PhaseType::Boss));

        // First boss dies
        handle_boss_death(&mut encounter, 100, 3000);
        assert_eq!(encounter.active_boss_ids.len(), 0);
        assert_eq!(encounter.current_phase, None); // Phase ends when all bosses dead

        // Second boss appears (new mob phase should start first)
        begin_mob_phase(&mut encounter, 3500);
        transition_to_boss_phase(&mut encounter, 200, 4000);
        assert_eq!(encounter.active_boss_ids.len(), 1);
        assert!(encounter.active_boss_ids.contains(&200));
    }

    #[test]
    fn test_phase_timeout() {
        let mut encounter = Encounter::default();

        // Start mob phase at t=1000
        begin_mob_phase(&mut encounter, 1000);
        encounter.time_last_combat_packet_ms = 1000;

        // Check timeout at t=10000 (9 seconds - should not timeout)
        assert!(!check_phase_timeout(&encounter, 10000));

        // Check timeout at t=20000 (19 seconds - should timeout)
        assert!(check_phase_timeout(&encounter, 20000));

        // No phase active - should not timeout
        encounter.current_phase = None;
        assert!(!check_phase_timeout(&encounter, 30000));
    }

    #[test]
    fn test_calculate_active_combat_time() {
        // Test with multiple phases
        let phases = vec![
            (1000, Some(5000)),  // 4 seconds
            (10000, Some(15000)), // 5 seconds
            (20000, Some(25000)), // 5 seconds
        ];

        let total = calculate_active_combat_time(&phases);
        assert_eq!(total, 14.0); // 4 + 5 + 5 = 14 seconds

        // Test with ongoing phase (no end time)
        let phases_with_ongoing = vec![
            (1000, Some(5000)),  // 4 seconds
            (10000, None),       // Ongoing - should be 0
        ];

        let total = calculate_active_combat_time(&phases_with_ongoing);
        assert_eq!(total, 4.0);
    }

    #[test]
    fn test_get_phase_duration() {
        // Completed phase
        assert_eq!(get_phase_duration(1000, Some(6000)), 5.0);

        // Ongoing phase (no end time)
        assert_eq!(get_phase_duration(1000, None), 0.0);

        // Same start and end (edge case)
        assert_eq!(get_phase_duration(5000, Some(5000)), 0.0);
    }

    #[test]
    fn test_should_split_mob_phase() {
        let mut encounter = Encounter::default();

        // Not in mob phase - should not split
        encounter.current_phase = Some(PhaseType::Boss);
        encounter.time_last_combat_packet_ms = 1000;
        assert!(!should_split_mob_phase(&encounter, 20000));

        // In mob phase but no timeout - should not split
        encounter.current_phase = Some(PhaseType::Mob);
        encounter.time_last_combat_packet_ms = 15000;
        assert!(!should_split_mob_phase(&encounter, 20000));

        // In mob phase with timeout - should split
        encounter.current_phase = Some(PhaseType::Mob);
        encounter.time_last_combat_packet_ms = 1000;
        assert!(should_split_mob_phase(&encounter, 20000));
    }
}
