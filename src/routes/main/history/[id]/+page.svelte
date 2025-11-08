<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { commands } from '$lib/bindings';
  import type { ActorEncounterStatDto, EncounterSummaryDto, SkillsWindow } from '$lib/bindings';
  import type { MetricType } from '$lib/api';
  import { getClassIcon, tooltip, CLASS_MAP } from '$lib/utils.svelte';
  import CrownIcon from "virtual:icons/lucide/crown";
  import TableRowGlow from '$lib/components/table-row-glow.svelte';
  import AbbreviatedNumber from '$lib/components/abbreviated-number.svelte';
  import { historyDpsPlayerColumns, historyDpsSkillColumns, historyHealPlayerColumns, historyHealSkillColumns } from '$lib/history-columns';
  import { settings, SETTINGS } from '$lib/settings-store';
  import getDisplayName from '$lib/name-display.ts';

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

  // Calculate max values for relative to top settings
  let maxDpsPlayer = $derived.by(() => {
    return displayedPlayers.reduce((max, p) => Math.max(max, p.totalDmg || 0), 0);
  });

  let maxHealPlayer = $derived.by(() => {
    return displayedPlayers.reduce((max, p) => Math.max(max, p.healDealt || 0), 0);
  });

  let maxDpsSkill = $derived.by(() => {
    if (!skillsWindow) return 0;
    return skillsWindow.skillRows.reduce((max, s) => Math.max(max, s.totalDmg || 0), 0);
  });

  let maxHealSkill = $derived.by(() => {
    if (!skillsWindow) return 0;
    return skillsWindow.skillRows.reduce((max, s) => Math.max(max, s.totalDmg || 0), 0);
  });

  // Get visible columns based on settings and active tab
  let visiblePlayerColumns = $derived.by(() => {
    if (activeTab === 'healing') {
      return historyHealPlayerColumns.filter(col => settings.state.history.heal.players[col.key]);
    }
    return historyDpsPlayerColumns.filter(col => settings.state.history.dps.players[col.key]);
  });

  let visibleSkillColumns = $derived.by(() => {
    if (skillType === 'heal') {
      return historyHealSkillColumns.filter(col => settings.state.history.heal.skillBreakdown[col.key]);
    }
    return historyDpsSkillColumns.filter(col => settings.state.history.dps.skillBreakdown[col.key]);
  });

  async function loadEncounter() {
    if (!encounterId) return;

    // Load encounter details
    const encounterRes = await commands.getEncounterById(encounterId);
    console.log("encounter res", encounterRes)
    if (encounterRes.status === 'ok') {
      encounter = encounterRes.data;
      actors = encounterRes.data.actors ?? [];
    } else {
      error = String(encounterRes.error);
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
      const bossCritTotal = a.bossCritTotalDealt || 0;
      const critTotal = a.critHitsDealt ? (bossOnlyMode ? bossCritTotal : (a.critTotalDealt || 0)) : 0;
      const bossLuckyTotal = a.bossLuckyTotalDealt || 0;
      const luckyTotal = a.luckyHitsDealt ? (bossOnlyMode ? bossLuckyTotal : (a.luckyTotalDealt || 0)) : 0;

      return {
        uid: a.actorId,
        name: a.name ?? String(a.actorId),
        isLocalPlayer: a.isLocalPlayer ?? false,
        className,
        classDisplay: className || "Unknown Class",
        abilityScore: a.abilityScore || 0,
        totalDmg: dmgValue,
        dps: dmgValue / durationSecs,
        dmgPct: totalDmgValue > 0 ? (dmgValue * 100) / totalDmgValue : 0,
        critRate: hits > 0 ? (a.critHitsDealt || 0) / hits : 0,
        critDmgRate: dmgValue > 0 ? critTotal / dmgValue : 0,
        luckyRate: hits > 0 ? (a.luckyHitsDealt || 0) / hits : 0,
        luckyDmgRate: dmgValue > 0 ? luckyTotal / dmgValue : 0,
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
    // Re-run when charId changes
    charId;
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
                <span>
                  {#each encounter.bosses as b, i}
                    <span class="{b.isDefeated ? 'text-red-400 line-through' : 'text-blue-400'}">{b.monsterName}{i < encounter.bosses.length - 1 ? ', ' : ''}</span>
                  {/each}
                </span>
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
            {#each visiblePlayerColumns as col (col.key)}
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">{col.header}</th>
            {/each}
          </tr>
        </thead>
        <tbody class="bg-neutral-900">
          {#each displayedPlayers as p (p.uid)}
            <tr
              class="relative border-t border-neutral-800 hover:bg-neutral-800 transition-colors cursor-pointer"
              onclick={() => viewPlayerSkills(p.uid, activeTab === 'healing' ? 'heal' : 'dps')}
            >
              <td class="px-3 py-3 text-sm text-neutral-300 relative z-10">
                <div class="flex items-center gap-2 h-full">
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
                    {#if p.abilityScore > 0}
                      {#if SETTINGS.history.general.state.shortenAbilityScore}
                        <span class="text-neutral-400"><AbbreviatedNumber num={p.abilityScore} /></span>
                      {:else}
                        <span class="text-neutral-400">{p.abilityScore}</span>
                      {/if}
                    {/if}
                    {getDisplayName({
                      player: { uid: p.uid, name: p.name, className: p.className },
                      showYourNameSetting: settings.state.history.general.showYourName,
                      showOthersNameSetting: settings.state.history.general.showOthersName,
                      isLocalPlayer: p.isLocalPlayer
                    })}
                    {#if p.isLocalPlayer}
                      <span class="text-blue-400 ml-1">(You)</span>
                    {/if}
                  </span>
                </div>
              </td>
              {#each visiblePlayerColumns as col (col.key)}
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">
                  {#if (col.key === 'totalDmg' || col.key === 'dps') && (
                    (activeTab === 'damage' && settings.state.history.general.shortenDps) ||
                    (activeTab === 'healing' && settings.state.history.general.shortenDps) ||
                    (activeTab === 'tanked' && settings.state.history.general.shortenDps)
                  )}
                    {#if SETTINGS.history.general.state.shortenDps}
                      <AbbreviatedNumber num={p[col.key] ?? 0} />
                    {:else}
                      {col.format(p[col.key] ?? 0)}
                    {/if}
                  {:else}
                    {col.format(p[col.key] ?? 0)}
                  {/if}
                </td>
              {/each}
              <TableRowGlow
                className={p.className}
                percentage={
                  activeTab === 'healing'
                    ? (SETTINGS.history.general.state.relativeToTopHealPlayer && maxHealPlayer > 0 ? (p.healDealt / maxHealPlayer) * 100 : p.healPct)
                    : (activeTab === 'tanked'
                      ? p.tankedPct
                      : (SETTINGS.history.general.state.relativeToTopDPSPlayer && maxDpsPlayer > 0 ? (p.totalDmg / maxDpsPlayer) * 100 : p.dmgPct))
                }
              />
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
            Player: {getDisplayName({
              player: { uid: selectedPlayer.uid, name: selectedPlayer.name, className: selectedPlayer.className },
              showYourNameSetting: settings.state.history.general.showYourName,
              showOthersNameSetting: settings.state.history.general.showOthersName,
              isLocalPlayer: selectedPlayer.isLocalPlayer
            })} <span class="text-neutral-500">#{selectedPlayer.uid}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="overflow-x-auto rounded border border-neutral-700">
      <table class="w-full border-collapse">
        <thead>
          <tr class="bg-neutral-800">
            <th class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-neutral-400">Skill</th>
            {#each visibleSkillColumns as col (col.key)}
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-neutral-400">{col.header}</th>
            {/each}
          </tr>
        </thead>
        <tbody class="bg-neutral-900">
          {#each skillsWindow.skillRows as s (s.name)}
            <tr class="relative border-t border-neutral-800 hover:bg-neutral-800 transition-colors">
              <td class="px-3 py-3 text-sm text-neutral-300 relative z-10">{s.name}</td>
              {#each visibleSkillColumns as col (col.key)}
                <td class="px-3 py-3 text-right text-sm text-neutral-300 relative z-10">
                  {#if (col.key === 'totalDmg' || col.key === 'dps') && SETTINGS.history.general.state.shortenDps}
                    <AbbreviatedNumber num={s[col.key] ?? 0} />
                  {:else}
                    {col.format(s[col.key] ?? 0)}
                  {/if}
                </td>
              {/each}
              <TableRowGlow
                className={selectedPlayer.className}
                percentage={
                  skillType === 'heal'
                    ? (SETTINGS.history.general.state.relativeToTopHealSkill && maxHealSkill > 0 ? (s.totalDmg / maxHealSkill) * 100 : s.dmgPct)
                    : (SETTINGS.history.general.state.relativeToTopDPSSkill && maxDpsSkill > 0 ? (s.totalDmg / maxDpsSkill) * 100 : s.dmgPct)
                }
              />
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
