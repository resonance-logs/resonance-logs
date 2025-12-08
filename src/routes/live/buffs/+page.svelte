<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    onEncounterUpdate,
    onResetEncounter,
    type HeaderInfo,
  } from "$lib/api";
  import {
    getLiveBuffs,
    type BuffInfoDto,
    type EntityBuffsDto,
  } from "$lib/api_buffs";
  import { SETTINGS } from "$lib/settings-store";
  import { tooltip } from "$lib/utils.svelte";

  let buffData: EntityBuffsDto[] = $state([]);
  let pollInterval: ReturnType<typeof setInterval>;
  let encounterUnlisten: (() => void) | null = null;
  let resetUnlisten: (() => void) | null = null;

  let fightElapsedMs = $state(0);
  let fightStartTimestampMs = $state(0);
  let isEncounterPaused = $state(false);

  async function updateBuffs() {
    try {
      buffData = await getLiveBuffs();
      buffData.sort((a, b) => a.entityName.localeCompare(b.entityName));
      // Keep a lightweight client timer in sync while polling
      if (fightStartTimestampMs > 0 && !isEncounterPaused) {
        const clientElapsed = Date.now() - fightStartTimestampMs;
        fightElapsedMs = Math.max(fightElapsedMs, clientElapsed);
      }
    } catch (e) {
      console.error("Failed to fetch buffs", e);
    }
  }

  function handleEncounterUpdate(info: HeaderInfo, paused: boolean) {
    fightElapsedMs = info.elapsedMs;
    fightStartTimestampMs = info.fightStartTimestampMs;
    isEncounterPaused = paused;
    if (info.fightStartTimestampMs === 0) {
      fightElapsedMs = 0;
    }
  }

  function resetEncounterTimers() {
    fightElapsedMs = 0;
    fightStartTimestampMs = 0;
    isEncounterPaused = false;
  }

  // Helper: group buff events by stackCount and compute per-stack stats
  function getBuffStacks(buff: BuffInfoDto) {
    const map = new Map<
      number,
      { stackCount: number; casts: number; totalDurationMs: number }
    >();
    for (const ev of buff.events || []) {
      const stackCount = ev.stackCount ?? 0;
      const entry = map.get(stackCount);
      if (entry) {
        entry.casts += 1;
        entry.totalDurationMs += ev.durationMs ?? 0;
      } else {
        map.set(stackCount, {
          stackCount,
          casts: 1,
          totalDurationMs: ev.durationMs ?? 0,
        });
      }
    }
    return Array.from(map.values()).sort((a, b) => a.stackCount - b.stackCount);
  }

  function formatUptime(totalDurationMs: number) {
    if (fightElapsedMs <= 0) return 0;
    const effectiveDuration = Math.min(totalDurationMs, fightElapsedMs);
    return Math.min(100, Math.round((effectiveDuration / fightElapsedMs) * 100));
  }

  onMount(() => {
    onEncounterUpdate((event) => {
      handleEncounterUpdate(event.payload.headerInfo, event.payload.isPaused);
    }).then((fn) => {
      encounterUnlisten = fn;
    });

    onResetEncounter(() => {
      resetEncounterTimers();
    }).then((fn) => {
      resetUnlisten = fn;
    });

    updateBuffs();
    pollInterval = setInterval(updateBuffs, 1000);
  });

  onDestroy(() => {
    clearInterval(pollInterval);
    if (encounterUnlisten) encounterUnlisten();
    if (resetUnlisten) resetUnlisten();
  });

  let tableSettings = $derived(SETTINGS.live.tableCustomization.state);
  let customThemeColors = $derived(SETTINGS.accessibility.state.customThemeColors);
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
            <td class="px-3 py-2 align-top tabular-nums font-medium" style="color: {customThemeColors.tableTextColor}; width: 200px;">
                {entity.entityName}
            </td>
            <td class="px-3 py-2 align-top">
                <div class="flex flex-wrap gap-2">
                    {#each entity.buffs as buff}
                      {@const buffStacks = getBuffStacks(buff)}
                      {#each buffStacks as stack}
                        <div
                          class="flex flex-col gap-0.5 bg-black/20 rounded px-2 py-1 text-xs min-w-[140px] max-w-[200px]"
                          use:tooltip={() => buff.buffNameLong ?? `ID: ${buff.buffId}`}
                        >
                          <span class="font-semibold text-foreground truncate">
                            {buffStacks.length > 1
                              ? `${buff.buffName} (${stack.stackCount})`
                              : buff.buffName}
                          </span>
                          <div class="flex items-center gap-1 text-muted-foreground text-[10px]">
                            <span>{formatUptime(stack.totalDurationMs)}%</span>
                            <span class="text-muted-foreground"></span>
                            <span>{stack.casts} casts</span>
                          </div>
                        </div>
                      {/each}
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
