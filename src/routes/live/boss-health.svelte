<script lang="ts">
  /**
   * @file This component displays the health of the current boss(es).
   */
  import { onMount } from "svelte";
  import { onEncounterUpdate, onResetEncounter, type HeaderInfo } from "$lib/api";
  import { tooltip } from "$lib/utils.svelte";
  import { settings } from "$lib/settings-store";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";

  let headerInfo: HeaderInfo = $state({
    totalDps: 0,
    totalDmg: 0,
    elapsedMs: 0,
    fightStartTimestampMs: 0,
    bosses: [
      // {
      //   uid: 1,
      //   currentHp: 20,
      //   maxHp: 50,
      //   name: "Boss Example"
      // }
    ],
    sceneId: null,
    sceneName: null,
    currentPhase: null,
  });

  // Track compact mode
  let density = $derived(settings.state.accessibility.density ?? "comfortable");

  onMount(() => {
    let encounterUnlisten: (() => void) | null = null;
    let resetUnlisten: (() => void) | null = null;

    onEncounterUpdate((event) => {
      headerInfo = event.payload.headerInfo;
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
  <div class="flex flex-col gap-1">
    {#each headerInfo.bosses as boss (boss.uid)}
      {@const hpPercent = boss.maxHp && boss.currentHp !== null ? Math.min(100, Math.max(0, (boss.currentHp / boss.maxHp) * 100)) : 0}
      <div class="flex items-center gap-1 whitespace-nowrap">
        <span class="{density === 'comfortable' ? 'text-base' : density === 'medium' ? 'text-xs' : 'text-[11px]'} truncate text-neutral-100 font-semibold tracking-tight" {@attach tooltip(() => boss.name)}>{boss.name + " -"}</span>
        <span class="{density === 'comfortable' ? 'text-base' : density === 'medium' ? 'text-xs' : 'text-[11px]'} tabular-nums font-semibold text-neutral-100">
          <AbbreviatedNumber num={boss.currentHp !== null ? boss.currentHp : 0} />
          {#if boss.maxHp}
            <span> / <AbbreviatedNumber num={boss.maxHp} /></span>
          {/if}
          <span class="text-rose-400 ml-1">({hpPercent.toFixed(1)}%)</span>
        </span>
      </div>
    {/each}
  </div>
{:else}
  <span class="{density === 'comfortable' ? 'text-base' : density === 'medium' ? 'text-sm' : 'text-xs'} text-neutral-500 font-medium italic">No Boss</span>
{/if}
