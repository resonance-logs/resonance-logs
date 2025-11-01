<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import { historyDpsPlayerColumns, historyDpsSkillColumns, historyHealPlayerColumns, historyHealSkillColumns } from "$lib/history-columns";
  import { SETTINGS } from "$lib/settings-store";

  const SETTINGS_CATEGORY = "history";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
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
