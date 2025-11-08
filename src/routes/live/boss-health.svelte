<script lang="ts">
  /**
   * @file This component displays the health of the current boss(es).
   */
  import { onMount } from "svelte";
  import { onEncounterUpdate, onResetEncounter, type HeaderInfo } from "$lib/api";
  import { tooltip } from "$lib/utils.svelte";
  import { settings } from "$lib/settings-store";

  let headerInfo: HeaderInfo = $state({
    totalDps: 0,
    totalDmg: 0,
    elapsedMs: 0,
    fightStartTimestampMs: 0,
    bosses: [
      {
        uid: 1,
        currentHp: 20,
        maxHp: 50,
        name: "Boss Example"
      }
    ],
    sceneId: null,
    sceneName: null,
  });

  // Track compact mode
  let density = $derived(settings.state.accessibility.density ?? "comfortable");

  onMount(() => {
    let encounterUnlisten: (() => void) | null = null;
    let resetUnlisten: (() => void) | null = null;

    onEncounterUpdate(() => {
      // headerInfo = event.payload.headerInfo;
    }).then((fn) => {
      encounterUnlisten = fn;
    });

    onResetEncounter(() => {
      headerInfo.bosses = [];
    }).then((fn) => {
      resetUnlisten = fn;
    });

    return () => {
      if (encounterUnlisten) encounterUnlisten();
      if (resetUnlisten) resetUnlisten();
    };
  });
</script>

{#if headerInfo.bosses.length > 0}
  <div class="rounded-md border border-neutral-800/60 bg-neutral-900/50 px-3 py-2">
    {#each headerInfo.bosses as boss (boss.uid)}
      {@const hpPercent = boss.maxHp && boss.currentHp !== null ? Math.min(100, Math.max(0, (boss.currentHp / boss.maxHp) * 100)) : 0}
      <div class="flex items-center gap-3">
        <span class="{density === 'comfortable' ? 'text-[13px]' : density === 'medium' ? 'text-[12px]' : 'text-[11px]'} truncate text-neutral-200 font-semibold tracking-tight" {@attach tooltip(() => boss.name)}>{boss.name}</span>
        {#if density === 'comfortable'}
          <span class="text-neutral-300 tabular-nums text-[12px] font-medium">
            {boss.currentHp !== null ? boss.currentHp.toLocaleString() : "?"}{boss.maxHp ? ` / ${boss.maxHp.toLocaleString()}` : ""}
            <span class="text-neutral-400 ml-1.5">({hpPercent.toFixed(1)}%)</span>
          </span>
        {:else}
          <span class="text-neutral-300 tabular-nums text-[11px] font-medium">
            {hpPercent.toFixed(1)}%
          </span>
        {/if}
      </div>
    {/each}
  </div>
{/if}
