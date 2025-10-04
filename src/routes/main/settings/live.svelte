<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import { dpsPlayersColumnDefs, dpsSkillsColumnDefs, healPlayersColumnDefs, healSkillsColumnDefs } from "$lib/table-info";
  import { settings } from "$lib/settings-store";

  const SETTINGS_CATEGORY = "live";
  const LIVE_SETTINGS = $derived(settings.state[SETTINGS_CATEGORY]);
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <h2 class="my-4 text-lg font-medium">DPS - Player</h2>
  {#each dpsPlayersColumnDefs.filter((col) => col.accessorKey) as col (col.accessorKey)}
    <SettingsSwitch bind:checked={LIVE_SETTINGS["dps"]["players"][col.accessorKey]} label={col.meta?.label ?? "LABEL MISSING"} description={col.meta?.description} />
  {/each}

  <h2 class="my-4 text-lg font-medium">DPS - Skill Breakdown</h2>
  {#each dpsSkillsColumnDefs.filter((col) => col.accessorKey) as col (col.accessorKey)}
    <SettingsSwitch bind:checked={LIVE_SETTINGS["dps"]["skillBreakdown"][col.accessorKey]} label={col.meta?.label ?? "LABEL MISSING"} description={col.meta?.description} />
  {/each}

  <h2 class="my-4 text-lg font-medium">Heal - Player</h2>
  {#each healPlayersColumnDefs.filter((col) => col.accessorKey) as col (col.accessorKey)}
    <SettingsSwitch bind:checked={LIVE_SETTINGS["heal"]["players"][col.accessorKey]} label={col.meta?.label ?? "LABEL MISSING"} description={col.meta?.description} />
  {/each}

  <h2 class="my-4 text-lg font-medium">Heal - Skill Breakdown</h2>
  {#each healSkillsColumnDefs.filter((col) => col.accessorKey) as col (col.accessorKey)}
    <SettingsSwitch bind:checked={LIVE_SETTINGS["heal"]["skillBreakdown"][col.accessorKey]} label={col.meta?.label ?? "LABEL MISSING"} description={col.meta?.description} />
  {/each}
</Tabs.Content>
