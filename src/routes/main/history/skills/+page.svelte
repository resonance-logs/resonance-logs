<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { commands } from '$lib/bindings';
  import type { SkillsWindow } from '$lib/bindings';

  const encounterIdParam = $page.url.searchParams.get('encounterId');
  const playerUidParam = $page.url.searchParams.get('playerUid');
  const skillTypeParam = $page.url.searchParams.get('skillType') ?? 'dps';

  const encounterId = encounterIdParam ? parseInt(encounterIdParam) : null;
  const playerUid = playerUidParam ? parseInt(playerUidParam) : null;

  let skillsWindow: SkillsWindow = { currPlayer: [], skillRows: [] };
  let error: string | null = null;

  async function load() {
    if (!encounterId || !playerUid) return;
    const res = await commands.getEncounterPlayerSkills(encounterId, playerUid, skillTypeParam);
    if (res.status === 'ok') {
      skillsWindow = res.data;
    } else {
      error = String(res.error);
    }
  }

  onMount(() => {
    load();
  });
</script>

<div class="p-4">
  {#if error}
    <div class="text-red-400">{error}</div>
  {/if}

  <h2 class="text-lg font-semibold">Skill breakdown</h2>
  {#if skillsWindow.currPlayer && skillsWindow.currPlayer.length > 0}
    <div class="mb-2">Player: {skillsWindow.currPlayer[0].name} (#{skillsWindow.currPlayer[0].uid})</div>
  {/if}

  <table class="w-full table-fixed border-collapse">
    <thead>
      <tr class="bg-neutral-900 text-left">
        <th class="p-2">Skill</th>
        <th class="p-2 text-right">Total</th>
        <th class="p-2 text-right">DPS</th>
        <th class="p-2 text-right">D%</th>
        <th class="p-2 text-right">Hits</th>
      </tr>
    </thead>
    <tbody>
      {#each skillsWindow.skillRows as s (s.name)}
        <tr class="hover:bg-neutral-800 border-b">
          <td class="p-2">{s.name}</td>
          <td class="p-2 text-right">{s.totalDmg}</td>
          <td class="p-2 text-right">{s.dps.toFixed(1)}</td>
          <td class="p-2 text-right">{s.dmgPct.toFixed(1)}%</td>
          <td class="p-2 text-right">{s.hits}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
