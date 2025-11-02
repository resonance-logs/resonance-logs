<script lang="ts">
  import { onMount } from "svelte";
  import { onEncounterUpdate, type HeaderInfo } from "$lib/api";
  import { tooltip } from "$lib/utils.svelte";
  import { settings } from "$lib/settings-store";

  let headerInfo: HeaderInfo = $state({
    totalDps: 0,
    totalDmg: 0,
    elapsedMs: 0,
    fightStartTimestampMs: 0,
    bosses: [],
  });

  // Track compact mode
  let isCompactMode = $derived(settings.state.accessibility.compactMode);

  onMount(() => {
    let encounterUnlisten: (() => void) | null = null;

    onEncounterUpdate((event) => {
      headerInfo = event.payload.headerInfo;
    }).then((fn) => {
      encounterUnlisten = fn;
    });

    return () => {
      if (encounterUnlisten) encounterUnlisten();
    };
  });
</script>

{#if headerInfo.bosses.length > 0}
  <div class="{isCompactMode ? 'mb-1' : 'mb-2'} flex flex-col {isCompactMode ? 'gap-1' : 'gap-2'} bg-neutral-900/60 {isCompactMode ? 'px-2 py-1.5' : 'px-4 py-3'} rounded-lg">
    {#each headerInfo.bosses as boss (boss.uid)}
      {@const hpPercent = boss.maxHp && boss.currentHp !== null ? Math.min(100, Math.max(0, (boss.currentHp / boss.maxHp) * 100)) : 0}
      <div class="flex items-center {isCompactMode ? 'gap-2' : 'gap-3'}">
        <span class="{isCompactMode ? 'min-w-24 text-[11px]' : 'min-w-32 text-base'} truncate text-neutral-200 font-semibold" {@attach tooltip(() => boss.name)}>{boss.name}</span>
        <div class="relative {isCompactMode ? 'h-1.5' : 'h-3'} flex-1 rounded-md bg-neutral-800/80 overflow-hidden shadow-inner">
          <div
            class="absolute inset-0 rounded-md transition-all duration-300 ease-out bg-gradient-to-r from-cyan-500 to-blue-500"
            style={`width: ${hpPercent}%; box-shadow: 0 0 ${isCompactMode ? '4px' : '8px'} rgba(34, 211, 238, 0.5)`}
          >
          </div>
        </div>
        {#if !isCompactMode}
          <span class="text-neutral-300 tabular-nums text-sm font-medium min-w-40 text-right">
            {boss.currentHp !== null ? boss.currentHp.toLocaleString() : "?"}{boss.maxHp ? ` / ${boss.maxHp.toLocaleString()}` : ""}
            <span class="text-neutral-400 ml-1.5">({hpPercent.toFixed(1)}%)</span>
          </span>
        {:else}
          <span class="text-neutral-300 tabular-nums text-[11px] font-medium min-w-12 text-right">
            {hpPercent.toFixed(1)}%
          </span>
        {/if}
      </div>
    {/each}
  </div>
{/if}
