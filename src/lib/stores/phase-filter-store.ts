/**
 * Phase Filter Store
 *
 * Manages phase selection state for encounter views.
 * Tracks which phase is currently selected (null = overall encounter stats).
 */

import { writable } from 'svelte/store';

export interface PhaseFilterState {
  selectedPhaseId: number | null; // null = overall encounter stats
}

function createPhaseFilterStore() {
  const { subscribe, set, update } = writable<PhaseFilterState>({
    selectedPhaseId: null
  });

  return {
    subscribe,

    /**
     * Select a specific phase by ID
     */
    selectPhase: (phaseId: number | null): void => {
      update(state => ({ ...state, selectedPhaseId: phaseId }));
    },

    /**
     * Clear phase selection (return to overall encounter view)
     */
    clearSelection: (): void => {
      update(state => ({ ...state, selectedPhaseId: null }));
    },

    /**
     * Reset the store to initial state
     */
    reset: (): void => {
      set({ selectedPhaseId: null });
    }
  };
}

export const phaseFilterStore = createPhaseFilterStore();
