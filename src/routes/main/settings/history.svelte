<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import SettingsSelect from "./settings-select.svelte";
  import { historyDpsPlayerColumns, historyDpsSkillColumns, historyHealPlayerColumns, historyHealSkillColumns, historyTankedPlayerColumns, historyTankedSkillColumns } from "$lib/history-columns";
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
    <!-- General Settings -->
  <div class="bg-popover/40 rounded-lg border border-border/50 overflow-hidden">
      <button
        type="button"
  class="w-full flex items-center justify-between px-4 py-3 hover:bg-popover/50 transition-colors"
        onclick={() => toggleSection('general')}
      >
  <h2 class="text-base font-semibold text-foreground">General Settings</h2>
  <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.general ? 'rotate-180' : ''}" />
      </button>
      {#if expandedSections.general}
        <div class="px-4 pb-3 space-y-1">
          <SettingsSelect bind:selected={SETTINGS.history.general.state.showYourName} values={["Show Your Name", "Show Your Class", "Hide Your Name"]} label="Show Your Name" description="Show Your Class = replace your name with your class" />
          <SettingsSelect bind:selected={SETTINGS.history.general.state.showOthersName} values={["Show Others' Name", "Show Others' Class", "Hide Others' Name"]} label="Show Others' Name" description="Show Others' Class = replace others' name with their class" />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.showYourAbilityScore} label="Your Ability Score" description="Show your ability score" />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.showOthersAbilityScore} label="Others' Ability Score" description="Show others' ability score" />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopDPSPlayer} label="Relative to Top DPS - Player" description="Color bars are relative to top DPS player instead of all players. Useful for 20 man or World Bosses." />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopDPSSkill} label="Relative to Top DPS - Skill" description="Color bars are relative to top DPS skill instead of all skills. Useful for 20 man or World Bosses." />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopHealPlayer} label="Relative to Top Heal - Player" description="Color bars are relative to top healing player instead of all players. Useful for 20 man or World Bosses." />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopHealSkill} label="Relative to Top - Skill" description="Color bars are relative to top healing skill instead of all skills. Useful for 20 man or World Bosses." />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopTankedPlayer} label="Relative to Top Tanked - Player" description="Color bars are relative to top tanked player instead of all players. Useful for 20 man or World Bosses." />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopTankedSkill} label="Relative to Top Tanked - Skill" description="Color bars are relative to top tanked skill instead of all skills. Useful for 20 man or World Bosses." />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.shortenTps} label="Shorten TPS Metrics" description="Show TPS values as 5k, 50k, etc." />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.shortenAbilityScore} label="Shorten Ability Score" description="Shortens the Ability Score" />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.shortenDps} label="Shorten DPS Metrics" description="Show DPS values as 5k, 50k, etc." />
          <SettingsSwitch bind:checked={SETTINGS.history.general.state.bossOnlyDps} label="Boss Only Damage" description="Only count damage dealt to boss monsters" />
        </div>
      {/if}
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
