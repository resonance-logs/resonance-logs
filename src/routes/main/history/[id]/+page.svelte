<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { commands } from '$lib/bindings';
  import type { ActorEncounterStatDto, EncounterSummaryDto, SkillsWindow } from '$lib/bindings';
  import { getClassIcon, getClassColor, tooltip, CLASS_MAP } from '$lib/utils.svelte';
  import CrownIcon from "virtual:icons/lucide/crown";
  import TableRowGlow from '$lib/components/table-row-glow.svelte';

  // Get encounter ID from URL params
  let encounterId = $derived($page.params.id ? parseInt($page.params.id) : null);
  let charId = $derived($page.url.searchParams.get('charId'));
  let skillType = $derived($page.url.searchParams.get('skillType') ?? 'dps');

  // Class mapping functions
  function getClassName(classId: number | null): string {
    if (!classId) return "";
    return CLASS_MAP[classId] ?? "";
  }

  let encounter = $state<EncounterSummaryDto | null>(null);
  let actors = $state<ActorEncounterStatDto[]>([]);
  let players = $state<any[]>([]);
  let error = $state<string | null>(null);

  // Tab state for encounter view
  let activeTab = $state<'damage' | 'tanked' | 'healing'>('damage');
  let bossOnlyMode = $state(false);

  // Skills view state
  let skillsWindow = $state<SkillsWindow | null>(null);
  let selectedPlayer = $state<any | null>(null);

  // Filtered and sorted players based on active tab
  let displayedPlayers = $derived.by(() => {
    if (activeTab === 'damage') {
      return [...players].sort((a, b) => b.totalDmg - a.totalDmg);
    } else if (activeTab === 'tanked') {
      return [...players]
        .filter(p => p.damageTaken > 0)
        .sort((a, b) => b.damageTaken - a.damageTaken);
    } else if (activeTab === 'healing') {
      return [...players]
        .filter(p => p.healDealt > 0)
        .sort((a, b) => b.healDealt - a.healDealt);
    }
    return players;
  });

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
    const totalBossDmg = actors.reduce((sum, a) => sum + (a.bossDamageDealt ?? 0), 0);
    const totalDamageTaken = actors.reduce((sum, a) => sum + (a.damageTaken ?? 0), 0);
    const totalHealing = actors.reduce((sum, a) => sum + (a.healDealt ?? 0), 0);
    const durationSecs = Math.max(1, ((encounter.endedAtMs ?? Date.now()) - encounter.startedAtMs) / 1000);

    players = actors.map(a => {
      const hits = a.hitsDealt || 0;
      const hitsTaken = a.hitsTaken || 0;
      const hitsHeal = a.hitsHeal || 0;
      const className = getClassName(a.classId);

      const dmgValue = bossOnlyMode ? (a.bossDamageDealt || 0) : (a.damageDealt || 0);
      const totalDmgValue = bossOnlyMode ? totalBossDmg : totalDmg;

      return {
        uid: a.actorId,
        name: a.name ?? String(a.actorId),
        className,
        classDisplay: className || "Unknown Class",
        totalDmg: dmgValue,
        dps: dmgValue / durationSecs,
        dmgPct: totalDmgValue > 0 ? (dmgValue * 100) / totalDmgValue : 0,
        critRate: hits > 0 ? (a.critHitsDealt || 0) / hits : 0,
        hits: hits,
        hitsPerMinute: hits / (durationSecs / 60.0),
        // Tanked stats
        damageTaken: a.damageTaken || 0,
        tankedPS: (a.damageTaken || 0) / durationSecs,
        tankedPct: totalDamageTaken > 0 ? ((a.damageTaken || 0) * 100) / totalDamageTaken : 0,
        critTakenRate: hitsTaken > 0 ? (a.critHitsTaken || 0) / hitsTaken : 0,
        hitsTaken: hitsTaken,
        // Healing stats
        healDealt: a.healDealt || 0,
        hps: (a.healDealt || 0) / durationSecs,
        healPct: totalHealing > 0 ? ((a.healDealt || 0) * 100) / totalHealing : 0,
        critHealRate: hitsHeal > 0 ? (a.critHitsHeal || 0) / hitsHeal : 0,
        hitsHeal: hitsHeal,
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
      console.log('skills', res)
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

  $effect(() => {
    // Reload encounter when bossOnlyMode changes
    if (bossOnlyMode !== undefined) {
      loadEncounter();
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
      <div class="flex items-center justify-between gap-3 mb-3">
        <div class="flex items-center gap-3">
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
            <h2 class="text-xl font-semibold text-neutral-200">
              Encounter #{encounter.id}
              {#if encounter.bosses.length > 0}
                <span class="text-neutral-400">—</span>
                <span class="text-blue-400">{encounter.bosses.join(', ')}</span>
              {/if}
            </h2>
            <div class="text-sm text-neutral-400">
              {new Date(encounter.startedAtMs).toLocaleString()} — Duration: {Math.floor(Math.max(1, ((encounter.endedAtMs ?? Date.now()) - encounter.startedAtMs) / 1000) / 60)}m
            </div>
          </div>
        </div>

        <!-- Tabs and Boss Only Toggle -->
        <div class="flex items-end gap-2 h-[48px]">
          <div class="flex rounded border border-neutral-700 bg-neutral-900">
            <button
              onclick={() => activeTab = 'damage'}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab === 'damage' ? 'bg-neutral-800 text-neutral-200' : 'text-neutral-400 hover:text-neutral-200'}"
            >
              Damage
            </button>
            <button
              onclick={() => activeTab = 'tanked'}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab === 'tanked' ? 'bg-neutral-800 text-neutral-200' : 'text-neutral-400 hover:text-neutral-200'}"
            >
              Tanked
            </button>
            <button
              onclick={() => activeTab = 'healing'}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab === 'healing' ? 'bg-neutral-800 text-neutral-200' : 'text-neutral-400 hover:text-neutral-200'}"
            >
              Healing
            </button>
          </div>

          <button
            onclick={() => {if (activeTab === 'damage') bossOnlyMode = !bossOnlyMode}}
            class="boss-only-toggle transition-colors p-1 {activeTab !== 'damage' ? 'opacity-30 cursor-not-allowed' : ''}"
            class:boss-only-active={bossOnlyMode && activeTab === 'damage'}
            title={activeTab !== 'damage' ? "Boss Damage Only (Only for Damage tab)" : bossOnlyMode ? "Boss Damage Only (Active)" : "Boss Damage Only"}
          >
            <CrownIcon class="w-[16px] h-[16px] mb-0.25"/>
          </button>
        </div>
      </div>
    </div>

    <div class="overflow-x-auto rounded border border-neutral-700">
      <table class="w-full border-collapse">
        <thead>
          <tr class="bg-neutral-800">
            <th class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-neutral-400">Player</th>
            {#if activeTab === 'damage'}
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">DMG</th>
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">DPS</th>
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">D%</th>
            {:else if activeTab === 'tanked'}
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">Tanked</th>
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">TPS</th>
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">T%</th>
            {:else if activeTab === 'healing'}
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">Healing</th>
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">HPS</th>
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">H%</th>
            {/if}
          </tr>
        </thead>
        <tbody class="bg-neutral-900">
          {#each displayedPlayers as p (p.uid)}
            <tr
              class="relative border-t border-neutral-800 hover:bg-neutral-800 transition-colors cursor-pointer"
              onclick={() => viewPlayerSkills(p.uid, activeTab === 'healing' ? 'heal' : 'dps')}
            >
              <td class="px-3 py-3 text-sm text-neutral-300 relative z-10">
                <div class="flex items-end gap-2 h-full">
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
              {#if activeTab === 'damage'}
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.totalDmg.toLocaleString()}</td>
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.dps.toFixed(1)}</td>
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.dmgPct.toFixed(1)}%</td>
                <TableRowGlow className={p.className} percentage={p.dmgPct} />
              {:else if activeTab === 'tanked'}
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.damageTaken.toLocaleString()}</td>
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.tankedPS.toFixed(1)}</td>
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.tankedPct.toFixed(1)}%</td>
                <TableRowGlow className={p.className} percentage={p.tankedPct} />
              {:else if activeTab === 'healing'}
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.healDealt.toLocaleString()}</td>
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.hps.toFixed(1)}</td>
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{p.healPct.toFixed(1)}%</td>
                <TableRowGlow className={p.className} percentage={p.healPct} />
              {/if}
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
            <tr class="relative border-t border-neutral-800 hover:bg-neutral-800 transition-colors">
              <td class="px-3 py-3 text-sm text-neutral-300 relative z-10">{s.name}</td>
              <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{String(s.totalDmg)}</td>
              <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{s.dps.toFixed(1)}</td>
              <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">{s.dmgPct.toFixed(1)}%</td>
              <TableRowGlow className={selectedPlayer.className} percentage={s.dmgPct} />
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="text-neutral-400">Loading...</div>
  {/if}
</div>

<style>
  .boss-only-toggle {
    transition: color 150ms ease;
  }

  .boss-only-toggle:hover {
    color: #facc15;
  }

  .boss-only-toggle.boss-only-active {
    color: #facc15;
  }
</style>
