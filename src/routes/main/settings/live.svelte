<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import { SETTINGS } from "$lib/settings-store";
  import { setDungeonSegmentsEnabled } from "$lib/api";
  import ChevronDown from "virtual:icons/lucide/chevron-down";
  import { liveDpsPlayerColumns, liveDpsSkillColumns, liveHealPlayerColumns, liveHealSkillColumns, liveTankedPlayerColumns, liveTankedSkillColumns } from "$lib/column-data";

  const SETTINGS_CATEGORY = "live";

  // Sync dungeon segments setting to backend when user actually changes it.
  // Avoid calling on initial mount because the settings layout mounts all tabs
  // at once which would trigger multiple backend invocations and can stall
  // the live emitter. Only invoke the backend after the component is mounted
  // and the value changes.
  import { onMount } from 'svelte';
  let _mounted = false;
  onMount(() => {
    _mounted = true;
  });

  $effect(() => {
    if (_mounted) {
      void setDungeonSegmentsEnabled(SETTINGS.live.general.state.dungeonSegmentsEnabled);
    }
  });

  // Collapsible section state - all collapsed by default
  let expandedSections = $state({
    general: false,
    dpsPlayers: false,
    dpsSkills: false,
    healPlayers: false,
    healSkills: false,
    tankedPlayers: false,
    tankedSkills: false,
  });

  function toggleSection(section: keyof typeof expandedSections) {
    expandedSections[section] = !expandedSections[section];
  }
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <div class="space-y-3">
    <!-- General Settings moved to Themes > Live -->
    <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <div class="px-4 py-3 text-sm text-muted-foreground">
        <p>General settings have been moved to <span class="font-medium text-foreground">Themes → Live</span>.</p>
      </div>
    </div>

    <!-- DPS - Player Settings -->
  <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
        onclick={() => toggleSection('dpsPlayers')}
      >
  <h2 class="text-base font-semibold text-foreground">DPS - Player Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.dpsPlayers ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.dpsPlayers}
        <div class="px-4 pb-3 space-y-1">
          {#each liveDpsPlayerColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.live.dps.players.state[col.key as keyof typeof SETTINGS.live.dps.players.state]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- DPS - Skill Breakdown Settings -->
  <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
        onclick={() => toggleSection('dpsSkills')}
      >
  <h2 class="text-base font-semibold text-foreground">DPS - Skill Breakdown Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.dpsSkills ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.dpsSkills}
        <div class="px-4 pb-3 space-y-1">
          {#each liveDpsSkillColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.live.dps.skillBreakdown.state[col.key as keyof typeof SETTINGS.live.dps.skillBreakdown.state]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Heal - Player Settings -->
  <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
        onclick={() => toggleSection('healPlayers')}
      >
  <h2 class="text-base font-semibold text-foreground">Heal - Player Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.healPlayers ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.healPlayers}
        <div class="px-4 pb-3 space-y-1">
          {#each liveHealPlayerColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.live.heal.players.state[col.key as keyof typeof SETTINGS.live.heal.players.state]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Heal - Skill Breakdown Settings -->
  <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
        onclick={() => toggleSection('healSkills')}
      >
  <h2 class="text-base font-semibold text-foreground">Heal - Skill Breakdown Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.healSkills ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.healSkills}
        <div class="px-4 pb-3 space-y-1">
          {#each liveHealSkillColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.live.heal.skillBreakdown.state[col.key as keyof typeof SETTINGS.live.heal.skillBreakdown.state]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Tanked - Player Settings -->
  <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
        onclick={() => toggleSection('tankedPlayers')}
      >
  <h2 class="text-base font-semibold text-foreground">Tanked - Player Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.tankedPlayers ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.tankedPlayers}
        <div class="px-4 pb-3 space-y-1">
          {#each liveTankedPlayerColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.live.tanked.players.state[col.key as keyof typeof SETTINGS.live.tanked.players.state]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Tanked - Skill Breakdown Settings -->
  <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
        onclick={() => toggleSection('tankedSkills')}
      >
  <h2 class="text-base font-semibold text-foreground">Tanked - Skill Breakdown Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.tankedSkills ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.tankedSkills}
        <div class="px-4 pb-3 space-y-1">
          {#each liveTankedSkillColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.live.tanked.skills.state[col.key as keyof typeof SETTINGS.live.tanked.skills.state]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>
  </div>
</Tabs.Content>
