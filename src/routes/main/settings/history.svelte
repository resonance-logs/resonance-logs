<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import { historyDpsPlayerColumns, historyDpsSkillColumns, historyHealPlayerColumns, historyHealSkillColumns, historyTankedPlayerColumns, historyTankedSkillColumns } from "$lib/column-data";
  import { SETTINGS } from "$lib/settings-store";
  import { setBossOnlyDps } from "$lib/api";
  import ChevronDown from "virtual:icons/lucide/chevron-down";

  const SETTINGS_CATEGORY = "history";

  // Sync boss damage setting to backend when user actually changes it.
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
      void setBossOnlyDps(SETTINGS.history.general.state.bossOnlyDps);
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
    <!-- General Settings moved to Themes > Main -->
    <div class="bg-popover/40 rounded-lg border border-border/50 overflow-hidden">
      <div class="px-4 py-3 text-sm text-muted-foreground">
        <p>General settings have been moved to <span class="font-medium text-foreground">Themes → Main</span>.</p>
      </div>
    </div>

    <!-- DPS - Player Settings -->
  <div class="bg-popover/40 rounded-lg border border-border/50 overflow-hidden">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-popover/50 transition-colors"
        onclick={() => toggleSection('dpsPlayers')}
      >
  <h2 class="text-base font-semibold text-foreground">DPS - Player Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.dpsPlayers ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.dpsPlayers}
        <div class="px-4 pb-3 space-y-1">
          {#each historyDpsPlayerColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.history.dps.players.state[col.key]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- DPS - Skill Breakdown Settings -->
  <div class="bg-popover/40 rounded-lg border border-border/50 overflow-hidden">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-popover/50 transition-colors"
        onclick={() => toggleSection('dpsSkills')}
      >
  <h2 class="text-base font-semibold text-foreground">DPS - Skill Breakdown Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.dpsSkills ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.dpsSkills}
        <div class="px-4 pb-3 space-y-1">
          {#each historyDpsSkillColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.history.dps.skillBreakdown.state[col.key]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Heal - Player Settings -->
  <div class="bg-popover/40 rounded-lg border border-border/50 overflow-hidden">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-popover/50 transition-colors"
        onclick={() => toggleSection('healPlayers')}
      >
  <h2 class="text-base font-semibold text-foreground">Heal - Player Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.healPlayers ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.healPlayers}
        <div class="px-4 pb-3 space-y-1">
          {#each historyHealPlayerColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.history.heal.players.state[col.key]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Heal - Skill Breakdown Settings -->
  <div class="bg-popover/40 rounded-lg border border-border/50 overflow-hidden">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-popover/50 transition-colors"
        onclick={() => toggleSection('healSkills')}
      >
  <h2 class="text-base font-semibold text-foreground">Heal - Skill Breakdown Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.healSkills ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.healSkills}
        <div class="px-4 pb-3 space-y-1">
          {#each historyHealSkillColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.history.heal.skillBreakdown.state[col.key]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Tanked - Player Settings -->
  <div class="bg-popover/40 rounded-lg border border-border/50 overflow-hidden">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-popover/50 transition-colors"
        onclick={() => toggleSection('tankedPlayers')}
      >
  <h2 class="text-base font-semibold text-foreground">Tanked - Player Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.tankedPlayers ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.tankedPlayers}
        <div class="px-4 pb-3 space-y-1">
          {#each historyTankedPlayerColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.history.tanked.players.state[col.key]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Tanked - Skill Breakdown Settings -->
  <div class="bg-popover/40 rounded-lg border border-border/50 overflow-hidden">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-popover/50 transition-colors"
        onclick={() => toggleSection('tankedSkills')}
      >
  <h2 class="text-base font-semibold text-foreground">Tanked - Skill Breakdown Columns</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.tankedSkills ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.tankedSkills}
        <div class="px-4 pb-3 space-y-1">
          {#each historyTankedSkillColumns as col (col.key)}
            <SettingsSwitch bind:checked={SETTINGS.history.tanked.skillBreakdown.state[col.key]} label={col.label} description={col.description} />
          {/each}
        </div>
      {/if}
    </div>
  </div>
</Tabs.Content>
