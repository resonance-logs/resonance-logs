<script lang="ts">
  import { onMount } from "svelte";
  import { onEncounterUpdate, type HeaderInfo } from "$lib/api";
  import { tooltip } from "$lib/utils.svelte";

  let headerInfo: HeaderInfo = $state({
    totalDps: 0,
    totalDmg: 0,
    elapsedMs: 0,
    fightStartTimestampMs: 0,
    bosses: [],
  });

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
  <div class="mb-2 flex flex-col gap-2 bg-neutral-900/60 px-4 py-3 rounded-lg">
    {#each headerInfo.bosses as boss (boss.uid)}
      {@const hpPercent = boss.maxHp && boss.currentHp !== null ? Math.min(100, Math.max(0, (boss.currentHp / boss.maxHp) * 100)) : 0}
      <div class="flex items-center gap-3">
        <span class="min-w-32 truncate text-neutral-200 font-semibold text-base" {@attach tooltip(() => boss.name)}>{boss.name}</span>
        <div class="relative h-3 flex-1 rounded-md bg-neutral-800/80 overflow-hidden shadow-inner">
          <div
            class="absolute inset-0 rounded-md transition-all duration-300 ease-out bg-gradient-to-r from-cyan-500 to-blue-500"
            style={`width: ${hpPercent}%; box-shadow: 0 0 8px rgba(34, 211, 238, 0.5)`}
          >
          </div>
        </div>
        <span class="text-neutral-300 tabular-nums text-sm font-medium min-w-40 text-right">
          {boss.currentHp !== null ? boss.currentHp.toLocaleString() : "?"}{boss.maxHp ? ` / ${boss.maxHp.toLocaleString()}` : ""}
          <span class="text-neutral-400 ml-1.5">({hpPercent.toFixed(1)}%)</span>
        </span>
      </div>
    {/each}
  </div>
{/if}
