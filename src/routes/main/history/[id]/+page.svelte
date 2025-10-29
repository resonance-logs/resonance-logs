<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { commands } from '$lib/bindings';
  import type { ActorEncounterStatDto, EncounterSummaryDto, SkillsWindow } from '$lib/bindings';
  import { getClassIcon, getClassColor, tooltip } from '$lib/utils.svelte';

  // Get encounter ID from URL params
  let encounterId = $derived($page.params.id ? parseInt($page.params.id) : null);
  let charId = $derived($page.url.searchParams.get('charId'));
  let skillType = $derived($page.url.searchParams.get('skillType') ?? 'dps');

  // Class mapping functions
  function getClassName(classId: number | null): string {
    if (!classId) return "";
    const classMap: Record<number, string> = {
      1: "Stormblade",
      2: "Frost Mage",
      4: "Wind Knight",
      5: "Verdant Oracle",
      9: "Heavy Guardian",
      11: "Marksman",
      12: "Shield Knight",
      13: "Beat Performer",
    };
    return classMap[classId] ?? "";
  }

  let encounter = $state<EncounterSummaryDto | null>(null);
  let actors = $state<ActorEncounterStatDto[]>([]);
  let players = $state<any[]>([]);
  let error = $state<string | null>(null);

  // Skills view state
  let skillsWindow = $state<SkillsWindow | null>(null);
  let selectedPlayer = $state<any | null>(null);

  async function loadEncounter() {
    if (!encounterId) return;

    // Load encounter details
    const encounterRes = await commands.getEncounterById(encounterId);
    console.log("encounter res", encounterRes)
    if (encounterRes.status === 'ok') {
      encounter = encounterRes.data;
    } else {
      error = String(encounterRes.error);
      return;
    }

    // Load actor stats
    const actorRes = await commands.getEncounterActorStats(encounterId);
    console.log("encounter res", actorRes)
    if (actorRes.status === 'ok') {
      actors = actorRes.data;
    } else {
      error = String(actorRes.error);
      return;
    }

    const totalDmg = actors.reduce((sum, a) => sum + (a.damageDealt ?? 0), 0);
    const durationSecs = Math.max(1, ((encounter.endedAtMs ?? Date.now()) - encounter.startedAtMs) / 1000);

    players = actors.map(a => {
      const hits = a.hitsDealt || 0;
      const className = getClassName(a.classId);
      return {
        uid: a.actorId,
        name: a.name ?? String(a.actorId),
        className,
        classDisplay: className || "Unknown Class",
        totalDmg: a.damageDealt || 0,
        dps: (a.damageDealt || 0) / durationSecs,
        dmgPct: totalDmg > 0 ? ((a.damageDealt || 0) * 100) / totalDmg : 0,
        critRate: hits > 0 ? (a.critHitsDealt || 0) / hits : 0,
        hits: hits,
        hitsPerMinute: hits / (durationSecs / 60.0),
      };
    });
  }

  async function loadPlayerSkills() {
    if (!encounterId || !charId) {
      skillsWindow = null;
      selectedPlayer = null;
      return;
    }

    const playerUid = parseInt(charId);
    const res = await commands.getEncounterPlayerSkills(encounterId, playerUid, skillType);
    if (res.status === 'ok') {
      skillsWindow = res.data;
      selectedPlayer = players.find(p => p.uid === playerUid);
    } else {
      error = String(res.error);
    }
  }

  function viewPlayerSkills(playerUid: number, type = 'dps') {
    goto(`/main/history/${encounterId}?charId=${playerUid}&skillType=${type}`);
  }

  function backToEncounter() {
    goto(`/main/history/${encounterId}`);
  }

  function backToHistory() {
    goto('/main/history');
  }

  $effect(() => {
    loadEncounter();
  });

  $effect(() => {
    if (charId) {
      loadPlayerSkills();
    } else {
      skillsWindow = null;
      selectedPlayer = null;
    }
  });
</script>

<div class="p-6">
  {#if error}
    <div class="text-red-400 mb-3">{error}</div>
  {/if}

  {#if !charId && encounter}
    <!-- Encounter Overview -->
    <div class="mb-4">
      <div class="flex items-center gap-3 mb-2">
        <button
          onclick={backToHistory}
          class="p-1.5 text-neutral-400 hover:text-neutral-200 transition-colors rounded hover:bg-neutral-800"
          aria-label="Back to history"
        >
          <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <div>
          <h2 class="text-xl font-semibold text-neutral-200">Encounter #{encounter.id}</h2>
          <div class="text-sm text-neutral-400">
            {new Date(encounter.startedAtMs).toLocaleString()} — Duration: {Math.floor(Math.max(1, ((encounter.endedAtMs ?? Date.now()) - encounter.startedAtMs) / 1000) / 60)}m
          </div>
        </div>
      </div>
    </div>

    <div class="overflow-x-auto rounded border border-neutral-700">
      <table class="w-full border-collapse">
        <thead>
          <tr class="bg-neutral-800">
            <th class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-neutral-400">Player</th>
            <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">DMG</th>
            <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">DPS</th>
            <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">D%</th>
          </tr>
        </thead>
        <tbody class="bg-neutral-900">
          {#each players as p (p.uid)}
            <tr
              class="relative border-t border-neutral-800 hover:bg-neutral-800 transition-colors cursor-pointer"
              onclick={() => viewPlayerSkills(p.uid, 'dps')}
            >
              <td class="px-3 py-3 text-sm text-neutral-300 relative z-10">
                <div class="flex items-center gap-2">
                  <img
                    class="size-5 object-contain"
                    src={getClassIcon(p.className)}
                    alt="Class icon"
                    {@attach tooltip(() => p.classDisplay || "Unknown Class")}
                  />
                  <span
                    class="truncate"
                    {@attach tooltip(() => `UID: #${p.uid}`)}
                  >
                    {p.name}
                  </span>
                </div>
              </td>
              <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.totalDmg.toLocaleString()}</td>
              <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.dps.toFixed(1)}</td>
              <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.dmgPct.toFixed(1)}%</td>
              <!-- Gradient glow effect -->
              <td
                class="absolute left-0 top-0 h-full pointer-events-none"
                style="background: linear-gradient(to right, {getClassColor(p.className)}, transparent); width: {p.dmgPct}%; opacity: 0.3;"
              ></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else if charId && skillsWindow && selectedPlayer}
    <!-- Player Skills View -->
    <div class="mb-4">
      <div class="flex items-center gap-3 mb-2">
        <button
          onclick={backToEncounter}
          class="p-1.5 text-neutral-400 hover:text-neutral-200 transition-colors rounded hover:bg-neutral-800"
          aria-label="Back to encounter"
        >
          <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <div>
          <h2 class="text-xl font-semibold text-neutral-200">Skill Breakdown</h2>
          <div class="text-sm text-neutral-400">
            Player: {selectedPlayer.name} <span class="text-neutral-500">#{selectedPlayer.uid}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="overflow-x-auto rounded border border-neutral-700">
      <table class="w-full border-collapse">
        <thead>
          <tr class="bg-neutral-800">
            <th class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-neutral-400">Skill</th>
            <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">Total</th>
            <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">DPS</th>
            <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">D%</th>
          </tr>
        </thead>
        <tbody class="bg-neutral-900">
          {#each skillsWindow.skillRows as s (s.name)}
            <tr class="border-t border-neutral-800 hover:bg-neutral-800 transition-colors">
              <td class="px-3 py-3 text-sm text-neutral-300">{s.name}</td>
              <td class="px-3 py-3 text-right text-sm text-neutral-300">{String(s.totalDmg)}</td>
              <td class="px-3 py-3 text-right text-sm text-neutral-300">{s.dps.toFixed(1)}</td>
              <td class="px-3 py-3 text-right text-sm text-neutral-300">{s.dmgPct.toFixed(1)}%</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="text-neutral-400">Loading...</div>
  {/if}
</div>
