/// Phase detection module for encounter mob/boss splitting.
///
/// Detects when to transition between mob and boss phases based on:
/// - Boss entity detection in the encounter
/// - Wipe scenarios during different phases
use crate::database::{DbTask, enqueue};
use crate::live::opcodes_models::{Encounter, PhaseType};
use log::info;

/// Checks if the encounter should transition to boss phase.
///
/// This is triggered when a boss entity is first detected in the encounter.
pub fn check_boss_phase_transition(encounter: &Encounter) -> bool {
    // Transition to boss phase if:
    // 1. We're currently in mob phase (or no phase)
    // 2. A boss has been detected that is not dead

    // If we are already in a boss phase, don't transition again
    if encounter.current_phase == Some(PhaseType::Boss) {
        return false;
    }

    // Check if any boss entity exists and is NOT dead
    // This allows re-entering boss phase for sequential bosses
    encounter
        .entity_uid_to_entity
        .iter()
        .any(|(uid, entity)| entity.is_boss() && !encounter.dead_boss_uids.contains(uid))
}

/// Begins a new phase for the current encounter.
pub fn begin_phase(encounter: &mut Encounter, phase_type: PhaseType, timestamp_ms: u128) {
    let phase_str = match phase_type {
        PhaseType::Mob => "mob",
        PhaseType::Boss => "boss",
        PhaseType::Idle => "idle",
    };

    info!(
        "Beginning {} phase at timestamp {}",
        phase_str, timestamp_ms
    );

    encounter.current_phase = Some(phase_type);
    encounter.phase_start_ms = timestamp_ms;

    enqueue(DbTask::BeginPhase {
        phase_type: phase_str.to_string(),
        start_time_ms: timestamp_ms as i64,
    });
}

/// Ends the current phase for the encounter.
pub fn end_phase(encounter: &mut Encounter, outcome: &str, timestamp_ms: u128) {
    if let Some(phase_type) = encounter.current_phase {
        let phase_str = match phase_type {
            PhaseType::Mob => "mob",
            PhaseType::Boss => "boss",
            PhaseType::Idle => "idle",
        };

        info!(
            "Ending {} phase with outcome '{}' at timestamp {}",
            phase_str, outcome, timestamp_ms
        );

        enqueue(DbTask::EndPhase {
            phase_type: phase_str.to_string(),
            end_time_ms: timestamp_ms as i64,
            outcome: outcome.to_string(),
        });

        encounter.current_phase = None;
    }
}

/// Handles the transition from mob to boss phase.
pub fn transition_to_boss_phase(encounter: &mut Encounter, timestamp_ms: u128) {
    // End mob phase with success
    if encounter.current_phase == Some(PhaseType::Mob) {
        end_phase(encounter, "success", timestamp_ms);
    }

    // Begin boss phase
    begin_phase(encounter, PhaseType::Boss, timestamp_ms);
    encounter.boss_detected = true;
}

/// Handles boss death scenarios.
pub fn handle_boss_death(encounter: &mut Encounter, timestamp_ms: u128) {
    if encounter.current_phase == Some(PhaseType::Boss) {
        // Only end the boss phase if ALL engaged bosses are dead.
        // This allows multi-boss encounters to continue until the last boss dies.
        let all_bosses_dead = encounter
            .engaged_boss_uids
            .iter()
            .all(|uid| encounter.dead_boss_uids.contains(uid));

        if all_bosses_dead {
            end_phase(encounter, "success", timestamp_ms);

            // Transition to Idle phase
            begin_phase(encounter, PhaseType::Idle, timestamp_ms);

            // Also set pause flag for compatibility with paus duration
            encounter.is_encounter_paused = true;
            encounter.pause_start_ms = Some(timestamp_ms);

            info!(
                "All engaged bosses dead. Transitioned to Idle phase and paused timer at {}",
                timestamp_ms
            );
        } else {
            info!("Boss died, but other bosses are still engaged. Continuing boss phase.");
        }
    }
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
            PhaseType::Idle => {
                // Wipe during idle phase: end idle phase with wipe outcome
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

        // Transition to boss phase
        transition_to_boss_phase(&mut encounter, 2000);
        assert_eq!(encounter.current_phase, Some(PhaseType::Boss));
        assert!(encounter.boss_detected);
        assert_eq!(encounter.phase_start_ms, 2000);

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
        transition_to_boss_phase(&mut encounter, 2000);
        assert_eq!(encounter.current_phase, Some(PhaseType::Boss));
        assert!(encounter.boss_detected);

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

        // Transition to boss phase
        transition_to_boss_phase(&mut encounter, 2000);
        assert!(encounter.boss_detected);

        // Should not detect boss phase transition again
        assert!(!check_boss_phase_transition(&encounter));
    }

    #[test]
    fn test_multi_boss_phase_continuation() {
        let mut encounter = Encounter::default();

        // Begin boss phase
        transition_to_boss_phase(&mut encounter, 1000);
        assert_eq!(encounter.current_phase, Some(PhaseType::Boss));

        // Simulate 2 engaged bosses
        encounter.engaged_boss_uids.insert(100);
        encounter.engaged_boss_uids.insert(101);

        // Boss 100 dies
        encounter.dead_boss_uids.insert(100);
        handle_boss_death(&mut encounter, 2000);

        // Phase should still be Boss because 101 is alive
        assert_eq!(encounter.current_phase, Some(PhaseType::Boss));

        // Boss 101 dies
        encounter.dead_boss_uids.insert(101);
        handle_boss_death(&mut encounter, 3000);

        // Phase should transition to Idle now
        assert_eq!(encounter.current_phase, Some(PhaseType::Idle));
    }
}
