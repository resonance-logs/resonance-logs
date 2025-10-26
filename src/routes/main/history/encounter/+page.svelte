<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { commands } from '$lib/bindings';
  import type { ActorEncounterStatDto, EncounterSummaryDto } from '$lib/bindings';

  let encounterId: number | null = null;
  let encounter: EncounterSummaryDto | null = null;
  let actors: ActorEncounterStatDto[] = [];
  let players: any[] = [];
  let error: string | null = null;

  $: {
    const p = $page.url.searchParams.get('encounterId');
    encounterId = p ? parseInt(p) : null;
  }

  async function load() {
    if (!encounterId) return;
    const encRes = await commands.getEncounterById(encounterId);
    if (encRes.status === 'ok') {
      encounter = encRes.data;
    } else {
      error = String(encRes.error);
      return;
    }

    const res = await commands.getEncounterActorStats(encounterId);
    if (res.status === 'ok') {
      actors = res.data;
    } else {
      error = String(res.error);
      return;
    }

    const totalDmg = actors.reduce((sum, a) => sum + (a.damageDealt ?? 0), 0);
    const durationSecs = Math.max(1, ((encounter?.endedAtMs ?? Date.now()) - encounter!.startedAtMs) / 1000);

    players = actors.map(a => {
      const hits = a.hitsDealt || 0;
      return {
        uid: a.actorId,
        name: a.name ?? String(a.actorId),
        totalDmg: a.damageDealt || 0,
        dps: (a.damageDealt || 0) / durationSecs,
        dmgPct: totalDmg > 0 ? ((a.damageDealt || 0) * 100) / totalDmg : 0,
        critRate: hits > 0 ? (a.critHitsDealt || 0) / hits : 0,
        hits: hits,
        hitsPerMinute: hits / (durationSecs / 60.0),
      };
    });
  }

  onMount(() => {
    load();
  });

  function viewSkills(playerUid: number, skillType = 'dps') {
    goto(`/main/history/skills?encounterId=${encounterId}&playerUid=${playerUid}&skillType=${skillType}`);
  }
</script>

<div class="p-4">
  {#if error}
    <div class="text-red-400">{error}</div>
  {/if}

  {#if encounter}
    <h2 class="text-lg font-semibold">Encounter #{encounter.id} — {new Date(encounter.startedAtMs).toLocaleString()}</h2>
    <div class="text-sm text-neutral-400 mb-3">Duration: {Math.floor(Math.max(1, ((encounter.endedAtMs ?? Date.now()) - encounter.startedAtMs) / 1000) / 60)}m</div>

    <table class="w-full table-fixed border-collapse">
      <thead>
        <tr class="bg-neutral-900 text-left">
          <th class="p-2">Player</th>
          <th class="p-2 text-right">DMG</th>
          <th class="p-2 text-right">DPS</th>
          <th class="p-2 text-right">D%</th>
          <th class="p-2 text-right">Hits</th>
          <th class="p-2">Skills</th>
        </tr>
      </thead>
      <tbody>
        {#each players as p (p.uid)}
          <tr class="hover:bg-neutral-800 border-b">
            <td class="p-2">{p.name} <span class="text-xs text-neutral-500">#{p.uid}</span></td>
            <td class="p-2 text-right">{p.totalDmg}</td>
            <td class="p-2 text-right">{p.dps.toFixed(1)}</td>
            <td class="p-2 text-right">{p.dmgPct.toFixed(1)}%</td>
            <td class="p-2 text-right">{p.hits}</td>
            <td class="p-2"><button class="px-2 py-1 bg-neutral-700 rounded" on:click={() => viewSkills(p.uid, 'dps')}>View DPS</button></td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <div>Loading encounter...</div>
  {/if}
</div>
