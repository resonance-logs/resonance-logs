<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { commands } from '$lib/bindings';
  import type { ActorEncounterStatDto, EncounterSummaryDto, SkillsWindow } from '$lib/bindings';
  import { getClassIcon, tooltip, CLASS_MAP } from '$lib/utils.svelte';
  import CrownIcon from "virtual:icons/lucide/crown";
  import TableRowGlow from '$lib/components/table-row-glow.svelte';
  import AbbreviatedNumber from '$lib/components/abbreviated-number.svelte';
  import { historyDpsPlayerColumns, historyDpsSkillColumns, historyHealPlayerColumns, historyHealSkillColumns, historyTankedPlayerColumns, historyTankedSkillColumns } from '$lib/history-columns';
  import { settings, SETTINGS } from '$lib/settings-store';
  import getDisplayName from '$lib/name-display';
  import { getModuleApiBaseUrl } from '$lib/stores/uploading';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { getEncounterSegments, type Segment } from '$lib/api';

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

  // Segment selection state
  let segments = $state<Segment[]>([]);
  let selectedSegmentId = $state<number | null>(null); // null = all segments
  let selectedSegment = $derived.by(() => segments.find((s) => s.id === selectedSegmentId) ?? null);

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
    } else if (activeTab === 'tanked') {
      return historyTankedPlayerColumns.filter(col => settings.state.history.tanked.players[col.key]);
    }
    return historyDpsPlayerColumns.filter(col => settings.state.history.dps.players[col.key]);
  });

  let visibleSkillColumns = $derived.by(() => {
    if (skillType === 'heal') {
      return historyHealSkillColumns.filter(col => settings.state.history.heal.skillBreakdown[col.key]);
    } else if (skillType === 'tanked') {
      return historyTankedSkillColumns.filter(col => settings.state.history.tanked.skillBreakdown[col.key]);
    }
    return historyDpsSkillColumns.filter(col => settings.state.history.dps.skillBreakdown[col.key]);
  });

  let maxTankedPlayer = $derived.by(() => {
    return displayedPlayers.reduce((max, p) => Math.max(max, p.damageTaken || 0), 0);
  });
  let maxTankedSkill = $derived.by(() => {
    if (!skillsWindow) return 0;
    return skillsWindow.skillRows.reduce((max, s) => Math.max(max, s.totalDmg || 0), 0);
  });

  const websiteBaseUrl = $derived.by(() => {
    const apiBase = getModuleApiBaseUrl();
    if (!apiBase) {
      return 'https://bpsr.app';
    }

    try {
      const url = new URL(apiBase);
      if (url.hostname.startsWith('api.')) {
        url.hostname = url.hostname.replace(/^api\./, '');
      }
      url.pathname = '';
      return url.toString().replace(/\/$/, '');
    } catch (err) {
      console.error('Failed to parse website URL from API base:', apiBase, err);
      return 'https://bpsr.app';
    }
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

    // Load dungeon segments
    try {
      segments = await getEncounterSegments(encounterId);
    } catch (e) {
      console.error('Failed to load segments:', e);
      segments = [];
    }

    const displayActors = actors;

    const totalDmg = displayActors.reduce((sum, a) => sum + (a.damageDealt ?? 0), 0);
    const totalBossDmg = displayActors.reduce((sum, a) => sum + (a.bossDamageDealt ?? 0), 0);
    const totalDamageTaken = displayActors.reduce((sum, a) => sum + (a.damageTaken ?? 0), 0);
    const totalHealing = displayActors.reduce((sum, a) => sum + (a.healDealt ?? 0), 0);

    // Calculate duration for overall encounter
    const durationSecs = Math.max(
      1,
      ((encounter.endedAtMs ?? Date.now()) - encounter.startedAtMs) / 1000
    );

    players = displayActors.map(a => {
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

  function handleSegmentSelect(event: Event) {
    const value = (event.target as HTMLSelectElement).value;
    selectedSegmentId = value ? Number(value) : null;
  }

  async function openEncounterOnWebsite() {
    if (!encounter || !encounter.remoteEncounterId) return;

    const url = `${websiteBaseUrl}/encounter/${encounter.remoteEncounterId}`;
    try {
      await openUrl(url);
    } catch (err) {
      console.error('Failed to open URL:', url, err);
    }
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

<div class="">
  {#if error}
    <div class="text-red-400 mb-3">{error}</div>
  {/if}

  {#if !charId && encounter}
    <!-- Encounter Overview -->
    <div class="mb-4">
      <div class="flex items-center justify-between gap-3 mb-3">
        <div class="flex items-center gap-3 flex-wrap">
          <button
            onclick={backToHistory}
            class="p-1.5 text-muted-foreground hover:text-foreground transition-colors rounded hover:bg-muted/40"
            aria-label="Back to history"
          >
            <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          <div>
            <h2 class="text-xl font-semibold text-foreground">
              Encounter #{encounter.id}
              {#if encounter.bosses.length > 0}
                <span class="text-muted-foreground">—</span>
                <span>
                  {#each encounter.bosses as b, i}
                    <span class="{b.isDefeated ? 'text-destructive line-through' : 'text-primary'}">{b.monsterName}{i < encounter.bosses.length - 1 ? ', ' : ''}</span>
                  {/each}
                </span>
              {/if}
            </h2>
            <div class="text-sm text-muted-foreground">
              {new Date(encounter.startedAtMs).toLocaleString()} — Duration: {Math.floor(Math.max(1, ((encounter.endedAtMs ?? Date.now()) - encounter.startedAtMs) / 1000) / 60)}m
            </div>
            <!-- Segments info -->
            {#if segments.length > 0}
              <div class="text-xs text-muted-foreground mt-2">
                <div class="font-semibold text-foreground mb-1">Dungeon Segments ({segments.length})</div>
                <div class="flex flex-wrap gap-2">
                  {#each segments as segment}
                    <button
                      class="inline-flex items-center gap-1 px-2 py-0.5 rounded border transition-colors
                        {segment.segmentType === 'boss' ? 'border-orange-500/30 bg-orange-500/10 text-orange-400' : 'border-slate-500/30 bg-slate-500/10 text-slate-400'}
                        {selectedSegmentId === segment.id ? 'ring-2 ring-primary/50' : ''}
                        hover:bg-opacity-20"
                      onclick={() => selectedSegmentId = selectedSegmentId === segment.id ? null : segment.id}
                    >
                      <span class="font-semibold">{segment.segmentType === 'boss' ? segment.bossName || 'Boss' : 'Trash'}</span>
                      <span class="text-muted-foreground">•</span>
                      <span>{Math.floor(((segment.endedAtMs ?? Date.now()) - segment.startedAtMs) / 1000)}s</span>
                      <span class="text-muted-foreground">•</span>
                      <AbbreviatedNumber num={segment.totalDamage} />
                    </button>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
          {#if encounter.remoteEncounterId}
            <button
              onclick={openEncounterOnWebsite}
              class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs rounded bg-primary/10 text-primary hover:bg-primary/20 transition-colors"
              title="Open this encounter on resonance-logs.com"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
              Open on website
            </button>
          {/if}
        </div>

        <!-- Tabs, Segment Selector, and Boss Only Toggle -->
        <div class="flex items-end gap-2 h-[48px]">
          <!-- Segment Selector -->
          {#if segments.length > 0}
            <select
              value={selectedSegmentId ?? ''}
              onchange={handleSegmentSelect}
              class="px-2 py-1 text-xs rounded border border-border bg-popover text-foreground transition-colors hover:bg-muted/40 cursor-pointer"
            >
              <option value=''>All Segments</option>
              {#each segments as segment}
                <option value={segment.id}>
                  {segment.segmentType === 'boss' ? segment.bossName || 'Boss Segment' : 'Trash Segment'}
                  ({Math.floor(((segment.endedAtMs ?? Date.now()) - segment.startedAtMs) / 1000)}s)
                </option>
              {/each}
            </select>
          {/if}

          <div class="flex rounded border border-border bg-popover">
            <button
              onclick={() => activeTab = 'damage'}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab === 'damage' ? 'bg-muted/40 text-foreground' : 'text-muted-foreground hover:text-foreground'}"
            >
              Damage
            </button>
            <button
              onclick={() => activeTab = 'tanked'}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab === 'tanked' ? 'bg-muted/40 text-foreground' : 'text-muted-foreground hover:text-foreground'}"
            >
              Tanked
            </button>
            <button
              onclick={() => activeTab = 'healing'}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab === 'healing' ? 'bg-muted/40 text-foreground' : 'text-muted-foreground hover:text-foreground'}"
            >
              Healing
            </button>
          </div>

          <button
            onclick={() => {if (activeTab === 'damage') bossOnlyMode = !bossOnlyMode}}
            class="boss-only-toggle transition-colors p-1 {activeTab !== 'damage' ? 'opacity-30 cursor-not-allowed' : 'hover:bg-muted/40 rounded'}"
            class:boss-only-active={bossOnlyMode && activeTab === 'damage'}
            title={activeTab !== 'damage' ? "Boss Damage Only (Only for Damage tab)" : bossOnlyMode ? "Boss Damage Only (Active)" : "Boss Damage Only"}
          >
            <CrownIcon class="w-[16px] h-[16px] mb-0.25"/>
          </button>
        </div>
      </div>
    </div>

    <div class="overflow-x-auto rounded border border-border/60 bg-card/30">
      <table class="w-full border-collapse">
        <thead>
          <tr class="bg-popover/60">
            <th class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground">Player</th>
            {#each visiblePlayerColumns as col (col.key)}
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-muted-foreground">{col.header}</th>
            {/each}
          </tr>
        </thead>
        <tbody class="bg-background/40">
          {#each displayedPlayers as p (p.uid)}
            <tr
              class="relative border-t border-border/40 hover:bg-muted/60 transition-colors {activeTab === 'tanked' ? 'cursor-default' : 'cursor-pointer'}"
              onclick={() => activeTab !== 'tanked' && viewPlayerSkills(p.uid, activeTab === 'healing' ? 'heal' : 'dps')}
            >
              <td class="px-3 py-3 text-sm text-muted-foreground relative z-10">
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
                        <span class="text-muted-foreground"><AbbreviatedNumber num={p.abilityScore} /></span>
                      {:else}
                        <span class="text-muted-foreground">{p.abilityScore}</span>
                      {/if}
                    {/if}
                    {getDisplayName({
                      player: { uid: p.uid, name: p.name, className: p.className },
                      showYourNameSetting: settings.state.history.general.showYourName,
                      showOthersNameSetting: settings.state.history.general.showOthersName,
                      isLocalPlayer: p.isLocalPlayer
                    })}
                    {#if p.isLocalPlayer}
                      <span class="ml-1 text-[oklch(0.65_0.1_250)]">(You)</span>
                    {/if}
                  </span>
                </div>
              </td>
              {#each visiblePlayerColumns as col (col.key)}
                <td class="px-3 py-3 text-right text-sm text-muted-foreground relative z-10">
                  {#if (
                      (activeTab === 'damage' && (col.key === 'totalDmg' || col.key === 'dps') && SETTINGS.history.general.state.shortenDps) ||
                      (activeTab === 'healing' && (col.key === 'healDealt' || col.key === 'hps') && SETTINGS.history.general.state.shortenDps) ||
                      (activeTab === 'tanked' && (col.key === 'damageTaken' || col.key === 'tankedPS') && SETTINGS.history.general.state.shortenTps)
                    )}
                    {#if (activeTab === 'tanked' ? SETTINGS.history.general.state.shortenTps : SETTINGS.history.general.state.shortenDps)}
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
                      ? (SETTINGS.history.general.state.relativeToTopTankedPlayer && maxTankedPlayer > 0 ? (p.damageTaken / maxTankedPlayer) * 100 : p.tankedPct)
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
          <h2 class="text-xl font-semibold text-foreground">Skill Breakdown</h2>
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

    <div class="overflow-x-auto rounded border border-border/60 bg-card/30">
      <table class="w-full border-collapse">
        <thead>
          <tr class="bg-popover/60">
            <th class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground">Skill</th>
            {#each visibleSkillColumns as col (col.key)}
              <th class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-muted-foreground">{col.header}</th>
            {/each}
          </tr>
        </thead>
        <tbody class="bg-background/40">
          {#each skillsWindow.skillRows as s (s.name)}
            <tr class="relative border-t border-border/40 hover:bg-muted/60 transition-colors">
              <td class="px-3 py-3 text-sm text-muted-foreground relative z-10">{s.name}</td>
              {#each visibleSkillColumns as col (col.key)}
                <td class="px-3 py-3 text-right text-sm text-muted-foreground relative z-10">
                  {#if (col.key === 'totalDmg' || col.key === 'dps') && (skillType === 'tanked' ? SETTINGS.history.general.state.shortenTps : SETTINGS.history.general.state.shortenDps)}
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
                    : (skillType === 'tanked'
                      ? (SETTINGS.history.general.state.relativeToTopTankedSkill && maxTankedSkill > 0 ? (s.totalDmg / maxTankedSkill) * 100 : s.dmgPct)
                      : (SETTINGS.history.general.state.relativeToTopDPSSkill && maxDpsSkill > 0 ? (s.totalDmg / maxDpsSkill) * 100 : s.dmgPct))
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

