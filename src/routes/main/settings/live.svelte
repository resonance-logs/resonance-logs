<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import SettingsSelect from "./settings-select.svelte";
  import { dpsPlayersColumnDefs, dpsSkillsColumnDefs, healPlayersColumnDefs, healSkillsColumnDefs } from "$lib/table-info";
  import { SETTINGS } from "$lib/settings-store";
  import { setBossOnlyDps } from "$lib/api";

  const SETTINGS_CATEGORY = "live";

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
      void setBossOnlyDps(SETTINGS.live.general.state.bossOnlyDps);
    }
  });
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <h2 class="my-4 text-lg font-medium">General</h2>
  <SettingsSelect bind:selected={SETTINGS.live.general.state.showYourName} values={["Show Your Name", "Show Your Class", "Hide Your Name"]} label="Show Your Name" description="Show Your Class = replace your name with your class" />
  <SettingsSelect bind:selected={SETTINGS.live.general.state.showOthersName} values={["Show Others' Name", "Show Others' Class", "Hide Others' Name"]} label="Show Others' Name" description="Show Others' Class = replace others' name with their class" />
  <SettingsSwitch bind:checked={SETTINGS.live.general.state.showYourAbilityScore} label="Your Ability Score" description="Show your ability score" />
  <SettingsSwitch bind:checked={SETTINGS.live.general.state.showOthersAbilityScore} label="Others' Ability Score" description="Show others' ability score" />
  <SettingsSwitch bind:checked={SETTINGS.live.general.state.relativeToTopDPSPlayer} label="Relative to Top DPS - Player" description="Color bars are relative to top DPS player instead of all players. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.live.general.state.relativeToTopDPSSkill} label="Relative to Top DPS - Skill" description="Color bars are relative to top DPS skill instead of all skills. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.live.general.state.relativeToTopHealPlayer} label="Relative to Top Heal - Player" description="Color bars are relative to top healing player instead of all players. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.live.general.state.relativeToTopHealSkill} label="Relative to Top - Skill" description="Color bars are relative to top healing skill instead of all skills. Useful for 20 man or World Bosses." />
  <SettingsSwitch bind:checked={SETTINGS.live.general.state.shortenAbilityScore} label="Shorten Ability Score" description="Shortens the Ability Score" />
  <SettingsSwitch bind:checked={SETTINGS.live.general.state.bossOnlyDps} label="Boss Only Damage" description="Only count damage dealt to boss monsters" />

  <h2 class="my-4 text-lg font-medium">DPS - Player</h2>
  {#each dpsPlayersColumnDefs.filter((col) => col.accessorKey) as col (col.accessorKey)}
    <SettingsSwitch bind:checked={SETTINGS.live.dps.players.state[col.accessorKey]} label={col.meta?.label ?? "LABEL MISSING"} description={col.meta?.description} />
  {/each}

  <h2 class="my-4 text-lg font-medium">DPS - Skill Breakdown</h2>
  {#each dpsSkillsColumnDefs.filter((col) => col.accessorKey) as col (col.accessorKey)}
    <SettingsSwitch bind:checked={SETTINGS.live.dps.skillBreakdown.state[col.accessorKey]} label={col.meta?.label ?? "LABEL MISSING"} description={col.meta?.description} />
  {/each}

  <h2 class="my-4 text-lg font-medium">Heal - Player</h2>
  {#each healPlayersColumnDefs.filter((col) => col.accessorKey) as col (col.accessorKey)}
    <SettingsSwitch bind:checked={SETTINGS.live.heal.players.state[col.accessorKey]} label={col.meta?.label ?? "LABEL MISSING"} description={col.meta?.description} />
  {/each}

  <h2 class="my-4 text-lg font-medium">Heal - Skill Breakdown</h2>
  {#each healSkillsColumnDefs.filter((col) => col.accessorKey) as col (col.accessorKey)}
    <SettingsSwitch bind:checked={SETTINGS.live.heal.skillBreakdown.state[col.accessorKey]} label={col.meta?.label ?? "LABEL MISSING"} description={col.meta?.description} />
  {/each}
</Tabs.Content>
