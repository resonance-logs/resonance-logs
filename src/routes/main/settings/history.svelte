<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import SettingsSelect from "./settings-select.svelte";
  import { historyDpsPlayerColumns, historyDpsSkillColumns, historyHealPlayerColumns, historyHealSkillColumns } from "$lib/history-columns";
  import { SETTINGS } from "$lib/settings-store";
  import { setBossOnlyDps } from "$lib/api";

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
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <h2 class="my-4 text-lg font-medium">General</h2>
  <SettingsSelect bind:selected={SETTINGS.history.general.state.showYourName} values={["Show Your Name", "Show Your Class", "Hide Your Name"]} label="Show Your Name" description="Show Your Class = replace your name with your class" />
  <SettingsSelect bind:selected={SETTINGS.history.general.state.showOthersName} values={["Show Others' Name", "Show Others' Class", "Hide Others' Name"]} label="Show Others' Name" description="Show Others' Class = replace others' name with their class" />
  <SettingsSwitch bind:checked={SETTINGS.history.general.state.showYourAbilityScore} label="Your Ability Score" description="Show your ability score" />
  <SettingsSwitch bind:checked={SETTINGS.history.general.state.showOthersAbilityScore} label="Others' Ability Score" description="Show others' ability score" />
  <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopDPSPlayer} label="Relative to Top DPS - Player" description="Color bars are relative to top DPS player instead of all players. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopDPSSkill} label="Relative to Top DPS - Skill" description="Color bars are relative to top DPS skill instead of all skills. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopHealPlayer} label="Relative to Top Heal - Player" description="Color bars are relative to top healing player instead of all players. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.history.general.state.relativeToTopHealSkill} label="Relative to Top - Skill" description="Color bars are relative to top healing skill instead of all skills. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.history.general.state.shortenAbilityScore} label="Shorten Ability Score" description="Shortens the Ability Score" />
  <SettingsSwitch bind:checked={SETTINGS.history.general.state.bossOnlyDps} label="Boss Only Damage" description="Only count damage dealt to boss monsters" />

  <h2 class="my-4 text-lg font-medium">DPS - Player</h2>
  {#each historyDpsPlayerColumns as col (col.key)}
    <SettingsSwitch bind:checked={SETTINGS.history.dps.players.state[col.key]} label={col.label} description={col.description} />
  {/each}

  <h2 class="my-4 text-lg font-medium">DPS - Skill Breakdown</h2>
  {#each historyDpsSkillColumns as col (col.key)}
    <SettingsSwitch bind:checked={SETTINGS.history.dps.skillBreakdown.state[col.key]} label={col.label} description={col.description} />
  {/each}

  <h2 class="my-4 text-lg font-medium">Heal - Player</h2>
  {#each historyHealPlayerColumns as col (col.key)}
    <SettingsSwitch bind:checked={SETTINGS.history.heal.players.state[col.key]} label={col.label} description={col.description} />
  {/each}

  <h2 class="my-4 text-lg font-medium">Heal - Skill Breakdown</h2>
  {#each historyHealSkillColumns as col (col.key)}
    <SettingsSwitch bind:checked={SETTINGS.history.heal.skillBreakdown.state[col.key]} label={col.label} description={col.description} />
  {/each}
</Tabs.Content>
