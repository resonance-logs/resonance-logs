<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getLiveBuffs, type EntityBuffsDto } from "$lib/api_buffs";
  import { SETTINGS } from "$lib/settings-store";
  import { tooltip } from "$lib/utils.svelte";

  let buffData: EntityBuffsDto[] = $state([]);
  let pollInterval: ReturnType<typeof setInterval>;

  async function updateBuffs() {
    try {
      buffData = await getLiveBuffs();
      buffData.sort((a, b) => a.entityName.localeCompare(b.entityName));
    } catch (e) {
      console.error("Failed to fetch buffs", e);
    }
  }

  onMount(() => {
    updateBuffs();
    pollInterval = setInterval(updateBuffs, 1000);
  });

  onDestroy(() => {
    clearInterval(pollInterval);
  });

  let tableSettings = $derived(SETTINGS.live.tableCustomization.state);
</script>

<div class="relative flex flex-col gap-2 overflow-hidden rounded-lg ring-1 ring-border/60 bg-card/30">
  <table class="w-full border-collapse overflow-hidden">
    <thead>
        <tr class="bg-popover/60" style="height: {tableSettings.tableHeaderHeight}px;">
          <th class="px-3 py-1 text-left font-medium uppercase tracking-wide" style="font-size: {tableSettings.tableHeaderFontSize}px; color: {tableSettings.tableHeaderTextColor};">Player</th>
          <th class="px-3 py-1 text-left font-medium uppercase tracking-wide" style="font-size: {tableSettings.tableHeaderFontSize}px; color: {tableSettings.tableHeaderTextColor};">Buffs</th>
        </tr>
    </thead>
    <tbody>
      {#each buffData as entity (entity.entityUid)}
        <tr class="bg-background/40 hover:bg-muted/60 transition-colors border-b border-border/20 last:border-0">
            <td class="px-3 py-2 align-top tabular-nums font-medium" style="color: {tableSettings.playerTextColor}; width: 200px;">
                {entity.entityName}
            </td>
            <td class="px-3 py-2 align-top">
                <div class="flex flex-wrap gap-2">
                    {#each entity.buffs as buff}
                        <div class="inline-flex flex-col bg-black/20 rounded px-2 py-1 text-xs" use:tooltip={() => `ID: ${buff.buffId}`}>
                            <span class="font-semibold text-foreground">{buff.buffName}</span>
                            <span class="text-muted-foreground text-[10px]">{buff.events.length} events</span>
                        </div>
                    {/each}
                    {#if entity.buffs.length === 0}
                        <span class="text-muted-foreground italic">No buffs recorded</span>
                    {/if}
                </div>
            </td>
        </tr>
      {/each}
       {#if buffData.length === 0}
         <tr>
             <td colspan="2" class="px-3 py-4 text-center text-muted-foreground italic">No buff data available</td>
         </tr>
       {/if}
    </tbody>
  </table>
</div>
