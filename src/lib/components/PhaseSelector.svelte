<script lang="ts">
  import type { EncounterPhaseDto } from '$lib/bindings';
  import { phaseFilterStore } from '$lib/stores/phase-filter-store';

  interface Props {
    phases: EncounterPhaseDto[];
    selectedPhaseId?: number | null;
    onPhaseChange?: (phaseId: number | null) => void;
  }

  let { phases, selectedPhaseId = $bindable(null), onPhaseChange }: Props = $props();

  // Format duration in seconds
  function formatDuration(startMs: number, endMs: number | null): string {
    const durationMs: number = (endMs ?? Date.now()) - startMs;
    const durationSecs: number = Math.floor(durationMs / 1000);

    if (durationSecs < 60) {
      return `${durationSecs}s`;
    }

    const minutes: number = Math.floor(durationSecs / 60);
    const seconds: number = durationSecs % 60;
    return `${minutes}m ${seconds}s`;
  }

  // Generate phase label with count
  function getPhaseLabel(phase: EncounterPhaseDto, index: number): string {
    const phaseCount: number = phases.slice(0, index + 1).filter(p => p.phaseType === phase.phaseType).length;

    if (phase.phaseType === 'boss') {
      return `Boss Phase ${phaseCount}`;
    } else {
      return `Mob Pack ${phaseCount}`;
    }
  }

  // Get outcome display properties
  function getOutcomeProps(outcome: string): { label: string; color: string } {
    switch (outcome) {
      case 'success':
        return { label: '✓', color: 'text-green-400' };
      case 'wipe':
        return { label: '✗', color: 'text-red-400' };
      default:
        return { label: '?', color: 'text-yellow-400' };
    }
  }

  // Get phase type color
  function getPhaseTypeColor(phaseType: string): string {
    return phaseType === 'boss'
      ? 'border-purple-500/30 bg-purple-500/10 text-purple-400'
      : 'border-blue-500/30 bg-blue-500/10 text-blue-400';
  }

  // Handle phase selection change
  function handleChange(event: Event): void {
    const target = event.target as HTMLSelectElement;
    const value = target.value;
    const newPhaseId = value === 'null' ? null : parseInt(value, 10);
    selectedPhaseId = newPhaseId;

    // Update store
    phaseFilterStore.selectPhase(newPhaseId);

    // Call optional callback
    if (onPhaseChange) {
      onPhaseChange(newPhaseId);
    }
  }
</script>

{#if phases && phases.length > 0}
  <div class="phase-selector-wrapper">
    <select
      value={selectedPhaseId ?? 'null'}
      onchange={handleChange}
      class="px-3 py-1.5 text-sm rounded border border-border bg-popover text-foreground transition-colors hover:bg-muted/40 cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/50"
      aria-label="Select phase"
    >
      <option value="null">All Phases (Overall)</option>
      {#each phases as phase, i (phase.id)}
        {@const phaseLabel = getPhaseLabel(phase, i)}
        {@const duration = formatDuration(phase.startTimeMs, phase.endTimeMs)}
        {@const outcomeProps = getOutcomeProps(phase.outcome)}

        <option value={phase.id}>
          {phaseLabel} — {duration} {outcomeProps.label}
        </option>
      {/each}
    </select>

    <!-- Phase indicators (visual timeline) -->
    <div class="flex gap-1.5 mt-2">
      {#each phases as phase, i (phase.id)}
        {@const phaseLabel = getPhaseLabel(phase, i)}
        {@const duration = formatDuration(phase.startTimeMs, phase.endTimeMs)}
        {@const outcomeProps = getOutcomeProps(phase.outcome)}
        {@const phaseTypeColor = getPhaseTypeColor(phase.phaseType)}
        {@const isSelected = selectedPhaseId === phase.id}

        <button
          onclick={() => {
            selectedPhaseId = phase.id;
            phaseFilterStore.selectPhase(phase.id);
            if (onPhaseChange) {
              onPhaseChange(phase.id);
            }
          }}
          class="inline-flex items-center gap-1.5 px-2 py-1 rounded border transition-all text-xs {phaseTypeColor} {isSelected ? 'ring-2 ring-primary/50 scale-105' : 'hover:scale-102 opacity-80 hover:opacity-100'}"
          title="{phaseLabel} — {duration} — Outcome: {phase.outcome}"
        >
          <span class="font-semibold">{phase.phaseType === 'boss' ? '👑' : '⚔️'}</span>
          <span class="text-xs">{duration}</span>
          <span class="{outcomeProps.color} font-bold">{outcomeProps.label}</span>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .phase-selector-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  select {
    min-width: 200px;
  }

  button {
    white-space: nowrap;
  }
</style>
