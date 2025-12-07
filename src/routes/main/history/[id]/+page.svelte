<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { commands } from "$lib/bindings";
  import type {
    ActorEncounterStatDto,
    EncounterSummaryDto,
    SkillsWindow,
  } from "$lib/bindings";
  import { getClassIcon, tooltip, CLASS_MAP } from "$lib/utils.svelte";
  import CrownIcon from "virtual:icons/lucide/crown";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import {
    historyDpsPlayerColumns,
    historyDpsSkillColumns,
    historyHealPlayerColumns,
    historyHealSkillColumns,
    historyTankedPlayerColumns,
    historyTankedSkillColumns,
  } from "$lib/column-data";
  import { settings, SETTINGS } from "$lib/settings-store";
  import getDisplayName from "$lib/name-display";
  import { getModuleApiBaseUrl } from "$lib/stores/uploading";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { getEncounterSegments, type Segment } from "$lib/api";
  import {
    getEncounterBuffs,
    type EncounterEntityBuffsDto,
    type EncounterBuffDto,
  } from "$lib/api_buffs";

  // Get encounter ID from URL params
  let encounterId = $derived(
    $page.params.id ? parseInt($page.params.id) : null,
  );
  let charId = $derived($page.url.searchParams.get("charId"));
  let skillType = $derived($page.url.searchParams.get("skillType") ?? "dps");

  // Class mapping functions
  function getClassName(classId: number | null): string {
    if (!classId) return "";
    return CLASS_MAP[classId] ?? "";
  }

  let encounter = $state<EncounterSummaryDto | null>(null);
  let actors = $state<ActorEncounterStatDto[]>([]);
  let players = $state<any[]>([]);
  let error = $state<string | null>(null);
  let isDeleting = $state(false);
  let showDeleteModal = $state(false);

  // Tab state for encounter view
  let activeTab = $state<"damage" | "tanked" | "healing" | "buffs">("damage");
  let bossOnlyMode = $state(false);
  let buffs = $state<EncounterEntityBuffsDto[]>([]);

  // Segment state - read-only in the UI
  let segments = $state<Segment[]>([]);

  // Skills view state
  let skillsWindow = $state<SkillsWindow | null>(null);
  let selectedPlayer = $state<any | null>(null);

  // Filtered and sorted players based on active tab
  let displayedPlayers = $derived.by(() => {
    if (activeTab === "damage") {
      return [...players].sort((a, b) => b.totalDmg - a.totalDmg);
    } else if (activeTab === "tanked") {
      return [...players]
        .filter((p) => p.damageTaken > 0)
        .sort((a, b) => b.damageTaken - a.damageTaken);
    } else if (activeTab === "healing") {
      return [...players]
        .filter((p) => p.healDealt > 0)
        .sort((a, b) => b.healDealt - a.healDealt);
    }
    return players;
  });

  // Calculate max values for relative to top settings
  let maxDpsPlayer = $derived.by(() => {
    return displayedPlayers.reduce(
      (max, p) => Math.max(max, p.totalDmg || 0),
      0,
    );
  });

  let maxHealPlayer = $derived.by(() => {
    return displayedPlayers.reduce(
      (max, p) => Math.max(max, p.healDealt || 0),
      0,
    );
  });

  let maxDpsSkill = $derived.by(() => {
    if (!skillsWindow) return 0;
    return skillsWindow.skillRows.reduce(
      (max, s) => Math.max(max, s.totalDmg || 0),
      0,
    );
  });

  let maxHealSkill = $derived.by(() => {
    if (!skillsWindow) return 0;
    return skillsWindow.skillRows.reduce(
      (max, s) => Math.max(max, s.totalDmg || 0),
      0,
    );
  });

  // Get visible columns based on settings and active tab
  let visiblePlayerColumns = $derived.by(() => {
    if (activeTab === "healing") {
      return historyHealPlayerColumns.filter(
        (col) => settings.state.history.heal.players[col.key],
      );
    } else if (activeTab === "tanked") {
      return historyTankedPlayerColumns.filter(
        (col) => settings.state.history.tanked.players[col.key],
      );
    }
    return historyDpsPlayerColumns.filter(
      (col) => settings.state.history.dps.players[col.key],
    );
  });

  let visibleSkillColumns = $derived.by(() => {
    if (skillType === "heal") {
      return historyHealSkillColumns.filter(
        (col) => settings.state.history.heal.skillBreakdown[col.key],
      );
    } else if (skillType === "tanked") {
      return historyTankedSkillColumns.filter(
        (col) => settings.state.history.tanked.skillBreakdown[col.key],
      );
    }
    return historyDpsSkillColumns.filter(
      (col) => settings.state.history.dps.skillBreakdown[col.key],
    );
  });

  let maxTankedPlayer = $derived.by(() => {
    return displayedPlayers.reduce(
      (max, p) => Math.max(max, p.damageTaken || 0),
      0,
    );
  });
  let maxTankedSkill = $derived.by(() => {
    if (!skillsWindow) return 0;
    return skillsWindow.skillRows.reduce(
      (max, s) => Math.max(max, s.totalDmg || 0),
      0,
    );
  });

  const websiteBaseUrl = $derived.by(() => {
    const apiBase = getModuleApiBaseUrl();
    if (!apiBase) {
      return "https://bpsr.app";
    }

    try {
      const url = new URL(apiBase);
      if (url.hostname.startsWith("api.")) {
        url.hostname = url.hostname.replace(/^api\./, "");
      }
      url.pathname = "";
      return url.toString().replace(/\/$/, "");
    } catch (err) {
      console.error("Failed to parse website URL from API base:", apiBase, err);
      return "https://bpsr.app";
    }
  });

  async function loadEncounter() {
    if (!encounterId) return;

    // Load encounter details
    const encounterRes = await commands.getEncounterById(encounterId);
    console.log("encounter res", encounterRes);
    if (encounterRes.status === "ok") {
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
      console.error("Failed to load segments:", e);
      segments = [];
    }

    const displayActors = actors;

    const totalDmg = displayActors.reduce(
      (sum, a) => sum + (a.damageDealt ?? 0),
      0,
    );
    const totalBossDmg = displayActors.reduce(
      (sum, a) => sum + (a.bossDamageDealt ?? 0),
      0,
    );
    const totalDamageTaken = displayActors.reduce(
      (sum, a) => sum + (a.damageTaken ?? 0),
      0,
    );
    const totalHealing = displayActors.reduce(
      (sum, a) => sum + (a.healDealt ?? 0),
      0,
    );

    // Calculate duration for overall encounter
    const durationSecs = Math.max(
      1,
      ((encounter.endedAtMs ?? Date.now()) - encounter.startedAtMs) / 1000,
    );

    players = displayActors.map((a) => {
      const hits = a.hitsDealt || 0;
      const hitsTaken = a.hitsTaken || 0;
      const hitsHeal = a.hitsHeal || 0;
      const className = getClassName(a.classId);

      const dmgValue = bossOnlyMode
        ? a.bossDamageDealt || 0
        : a.damageDealt || 0;
      const totalDmgValue = bossOnlyMode ? totalBossDmg : totalDmg;
      const bossCritTotal = a.bossCritTotalDealt || 0;
      const critTotal = a.critHitsDealt
        ? bossOnlyMode
          ? bossCritTotal
          : a.critTotalDealt || 0
        : 0;
      const bossLuckyTotal = a.bossLuckyTotalDealt || 0;
      const luckyTotal = a.luckyHitsDealt
        ? bossOnlyMode
          ? bossLuckyTotal
          : a.luckyTotalDealt || 0
        : 0;

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
        tankedPct:
          totalDamageTaken > 0
            ? ((a.damageTaken || 0) * 100) / totalDamageTaken
            : 0,
        critTakenRate: hitsTaken > 0 ? (a.critHitsTaken || 0) / hitsTaken : 0,
        hitsTaken: hitsTaken,
        // Healing stats
        healDealt: a.healDealt || 0,
        hps: (a.healDealt || 0) / durationSecs,
        healPct:
          totalHealing > 0 ? ((a.healDealt || 0) * 100) / totalHealing : 0,
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
    const res = await commands.getEncounterPlayerSkills(
      encounterId,
      playerUid,
      skillType,
    );
    if (res.status === "ok") {
      console.log("skills", res);
      skillsWindow = res.data;
      selectedPlayer = players.find((p) => p.uid === playerUid);
    } else {
      error = String(res.error);
    }
  }

  function viewPlayerSkills(playerUid: number, type = "dps") {
    goto(`/main/history/${encounterId}?charId=${playerUid}&skillType=${type}`);
  }

  function backToEncounter() {
    goto(`/main/history/${encounterId}`);
  }

  function backToHistory() {
    goto("/main/history");
  }

  // Segments are now read-only in the UI; selection is disabled

  async function handleToggleFavorite() {
    if (!encounter) return;
    try {
      const newStatus = !encounter.isFavorite;
      // Optimistic update
      encounter.isFavorite = newStatus;
      await commands.toggleFavoriteEncounter(encounter.id, newStatus);
    } catch (e) {
      console.error("Failed to toggle favorite", e);
      // Revert on error
      if (encounter) encounter.isFavorite = !encounter.isFavorite;
    }
  }

  function openDeleteModal() {
    showDeleteModal = true;
  }

  function closeDeleteModal() {
    showDeleteModal = false;
  }

  async function confirmDeleteEncounter() {
    if (!encounter) return;
    isDeleting = true;
    try {
      await commands.deleteEncounter(encounter.id);
      // Navigate back to history after deletion
      goto("/main/history");
    } catch (e) {
      console.error("Failed to delete encounter", e);
      alert("Failed to delete encounter: " + e);
      isDeleting = false;
      showDeleteModal = false;
    }
  }

  async function openEncounterOnWebsite() {
    if (!encounter || !encounter.remoteEncounterId) return;

    const url = `${websiteBaseUrl}/encounter/${encounter.remoteEncounterId}`;
    try {
      await openUrl(url);
    } catch (err) {
      console.error("Failed to open URL:", url, err);
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

  async function loadBuffs() {
    if (!encounterId) return;
    try {
      buffs = await getEncounterBuffs(encounterId);
      buffs.sort((a, b) => a.entityName.localeCompare(b.entityName));
    } catch (e) {
      console.error("Failed to load buffs", e);
    }
  }

  // Helper: group buff events by stackCount and compute per-stack stats
  function getBuffStacks(buff: EncounterBuffDto) {
    const map = new Map<number, { stackCount: number; casts: number; totalDurationMs: number }>();
    for (const ev of buff.events || []) {
      const sc = ev.stackCount ?? 0;
      const entry = map.get(sc);
      if (entry) {
        entry.casts += 1;
        entry.totalDurationMs += ev.durationMs ?? 0;
      } else {
        map.set(sc, { stackCount: sc, casts: 1, totalDurationMs: ev.durationMs ?? 0 });
      }
    }
    // Return as array sorted by stackCount ascending
    return Array.from(map.values()).sort((a, b) => a.stackCount - b.stackCount);
  }

  $effect(() => {
    if (encounterId && activeTab === "buffs") {
      loadBuffs();
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
            <svg
              class="w-5 h-5"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M15 19l-7-7 7-7"
              />
            </svg>
          </button>
          <div>
            <h2 class="text-xl font-semibold text-foreground">
              Encounter #{encounter.id}
              {#if encounter.bosses.length > 0}
                <span class="text-muted-foreground">—</span>
                <span>
                  {#each encounter.bosses as b, i}
                    <span
                      class={b.isDefeated
                        ? "text-destructive line-through"
                        : "text-primary"}
                      >{b.monsterName}{i < encounter.bosses.length - 1
                        ? ", "
                        : ""}</span
                    >
                  {/each}
                </span>
              {/if}
            </h2>
            <div class="text-sm text-muted-foreground">
              {new Date(encounter.startedAtMs).toLocaleString()} — Duration: {Math.floor(
                Math.max(
                  1,
                  ((encounter.endedAtMs ?? Date.now()) -
                    encounter.startedAtMs) /
                    1000,
                ) / 60,
              )}m
            </div>
            <!-- Segments info -->
            {#if segments.length > 0}
              <div class="text-xs text-muted-foreground mt-2">
                <div class="font-semibold text-foreground mb-1">
                  Dungeon Segments ({segments.length})
                </div>
                <div class="flex flex-wrap gap-2">
                  {#each segments as segment}
                    <span
                      class="inline-flex items-center gap-1 px-2 py-0.5 rounded border
                        {segment.segmentType === 'boss'
                        ? 'border-orange-500/30 bg-orange-500/10 text-orange-400'
                        : 'border-slate-500/30 bg-slate-500/10 text-slate-400'}"
                    >
                      <span class="font-semibold"
                        >{segment.segmentType === "boss"
                          ? segment.bossName || "Boss"
                          : "Trash"}</span
                      >
                      <span class="text-muted-foreground">•</span>
                      <span
                        >{Math.floor(
                          ((segment.endedAtMs ?? Date.now()) -
                            segment.startedAtMs) /
                            1000,
                        )}s</span
                      >
                      <span class="text-muted-foreground">•</span>
                      <AbbreviatedNumber num={segment.totalDamage} />
                    </span>
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
              <svg
                class="w-3.5 h-3.5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                />
              </svg>
              Open on website
            </button>
          {/if}

          <!-- Favorite Toggle -->
          <button
            onclick={handleToggleFavorite}
            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs rounded transition-colors {encounter.isFavorite
              ? 'bg-yellow-500/10 text-yellow-500 hover:bg-yellow-500/20'
              : 'bg-muted/40 text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
            title={encounter.isFavorite
              ? "Remove from favorites"
              : "Add to favorites"}
          >
            <svg
              class="w-3.5 h-3.5"
              fill={encounter.isFavorite ? "currentColor" : "none"}
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"
              />
            </svg>
            {encounter.isFavorite ? "Favorited" : "Favorite"}
          </button>

          <!-- Delete Button -->
          <button
            onclick={openDeleteModal}
            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs rounded bg-destructive/10 text-destructive hover:bg-destructive/20 transition-colors"
            title="Delete this encounter"
          >
            <svg
              class="w-3.5 h-3.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
              />
            </svg>
            Delete
          </button>
        </div>

        <!-- Tabs, Segment Selector, and Boss Only Toggle -->
        <div class="flex items-end gap-2 h-[48px]">
          <!-- Segment selector removed (display only) -->

          <div class="flex rounded border border-border bg-popover">
            <button
              onclick={() => (activeTab = "damage")}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab ===
              'damage'
                ? 'bg-muted/40 text-foreground'
                : 'text-muted-foreground hover:text-foreground'}"
            >
              Damage
            </button>
            <button
              onclick={() => (activeTab = "tanked")}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab ===
              'tanked'
                ? 'bg-muted/40 text-foreground'
                : 'text-muted-foreground hover:text-foreground'}"
            >
              Tanked
            </button>
            <button
              onclick={() => (activeTab = "healing")}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab ===
              'healing'
                ? 'bg-muted/40 text-foreground'
                : 'text-muted-foreground hover:text-foreground'}"
            >
              Healing
            </button>
            <button
              onclick={() => (activeTab = "buffs")}
              class="px-2 py-1 text-xs rounded transition-colors {activeTab ===
              'buffs'
                ? 'bg-muted/40 text-foreground'
                : 'text-muted-foreground hover:text-foreground'}"
            >
              Buffs
            </button>
          </div>

          <button
            onclick={() => {
              if (activeTab === "damage") bossOnlyMode = !bossOnlyMode;
            }}
            class="boss-only-toggle transition-colors p-1 {activeTab !==
            'damage'
              ? 'opacity-30 cursor-not-allowed'
              : 'hover:bg-muted/40 rounded'}"
            class:boss-only-active={bossOnlyMode && activeTab === "damage"}
            title={activeTab !== "damage"
              ? "Boss Damage Only (Only for Damage tab)"
              : bossOnlyMode
                ? "Boss Damage Only (Active)"
                : "Boss Damage Only"}
          >
            <CrownIcon class="w-[16px] h-[16px] mb-0.25" />
          </button>
        </div>
      </div>
    </div>

    {#if activeTab === "buffs"}
      <div
        class="overflow-x-auto rounded border border-border/60 bg-card/30 p-0"
      >
        <table class="w-full border-collapse">
          <thead>
            <tr class="bg-popover/60">
              <th
                class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground"
                >Player</th
              >
              <th
                class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground"
                >Buffs</th
              >
            </tr>
          </thead>
          <tbody class="bg-background/40">
            {#each buffs as entity (entity.entityUid)}
              <tr
                class="border-t border-border/40 hover:bg-muted/60 transition-colors"
              >
                <td
                  class="px-3 py-3 text-sm text-foreground align-top w-[200px] font-medium"
                >
                  {entity.entityName}
                </td>
                <td class="px-3 py-3 text-sm text-foreground align-top">
                  <div class="flex flex-wrap gap-2">
                    {#each entity.buffs as buff}
                      {#each getBuffStacks(buff) as s}
                        <div
                          class="inline-flex flex-col bg-black/20 rounded px-2 py-1 text-xs"
                          {@attach tooltip(() => buff.buffNameLong ?? "")}
                        >
                          <span class="font-semibold text-foreground">{getBuffStacks(buff).length > 1 ? `${buff.buffName} - ${s.stackCount}` : buff.buffName}</span>
                          <span class="text-muted-foreground text-[10px]">
                            {#if encounter && encounter.startedAtMs}
                              {Math.round((s.totalDurationMs / Math.max(1, ((encounter.endedAtMs ?? Date.now()) - encounter.startedAtMs))) * 100)}% uptime • {s.casts} casts
                            {:else}
                              {Math.round((s.totalDurationMs / Math.max(1, (encounter?.duration ?? 1))) * 100)}% uptime • {s.casts} casts
                            {/if}
                          </span>
                        </div>
                      {/each}
                    {/each}
                    {#if entity.buffs.length === 0}
                      <span class="text-muted-foreground italic">No buffs</span>
                    {/if}
                  </div>
                </td>
              </tr>
            {/each}
            {#if buffs.length === 0}
              <tr
                ><td
                  colspan="2"
                  class="px-3 py-4 text-center text-muted-foreground italic"
                  >No buff data available</td
                ></tr
              >
            {/if}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="overflow-x-auto rounded border border-border/60 bg-card/30">
        <table class="w-full border-collapse">
          <thead>
            <tr class="bg-popover/60">
              <th
                class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground"
                >Player</th
              >
              {#each visiblePlayerColumns as col (col.key)}
                <th
                  class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-muted-foreground"
                  >{col.header}</th
                >
              {/each}
            </tr>
          </thead>
          <tbody class="bg-background/40">
            {#each displayedPlayers as p (p.uid)}
              <tr
                class="relative border-t border-border/40 hover:bg-muted/60 transition-colors {activeTab ===
                'tanked'
                  ? 'cursor-default'
                  : 'cursor-pointer'}"
                onclick={() =>
                  activeTab !== "tanked" &&
                  viewPlayerSkills(
                    p.uid,
                    activeTab === "healing" ? "heal" : "dps",
                  )}
              >
                <td
                  class="px-3 py-3 text-sm text-muted-foreground relative z-10"
                >
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
                          <span class="text-muted-foreground"
                            ><AbbreviatedNumber num={p.abilityScore} /></span
                          >
                        {:else}
                          <span class="text-muted-foreground"
                            >{p.abilityScore}</span
                          >
                        {/if}
                      {/if}
                      {getDisplayName({
                        player: {
                          uid: p.uid,
                          name: p.name,
                          className: p.className,
                          classSpecName: p.classSpecName,
                        },
                        showYourNameSetting:
                          settings.state.history.general.showYourName,
                        showOthersNameSetting:
                          settings.state.history.general.showOthersName,
                        isLocalPlayer: p.isLocalPlayer,
                      })}
                      {#if p.isLocalPlayer}
                        <span class="ml-1 text-[oklch(0.65_0.1_250)]"
                          >(You)</span
                        >
                      {/if}
                    </span>
                  </div>
                </td>
                {#each visiblePlayerColumns as col (col.key)}
                  <td
                    class="px-3 py-3 text-right text-sm text-muted-foreground relative z-10"
                  >
                    {#if (activeTab === "damage" && (col.key === "totalDmg" || col.key === "dps") && SETTINGS.history.general.state.shortenDps) || (activeTab === "healing" && (col.key === "healDealt" || col.key === "hps") && SETTINGS.history.general.state.shortenDps) || (activeTab === "tanked" && (col.key === "damageTaken" || col.key === "tankedPS") && SETTINGS.history.general.state.shortenTps)}
                      {#if activeTab === "tanked" ? SETTINGS.history.general.state.shortenTps : SETTINGS.history.general.state.shortenDps}
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
                  percentage={activeTab === "healing"
                    ? SETTINGS.history.general.state.relativeToTopHealPlayer &&
                      maxHealPlayer > 0
                      ? (p.healDealt / maxHealPlayer) * 100
                      : p.healPct
                    : activeTab === "tanked"
                      ? SETTINGS.history.general.state
                          .relativeToTopTankedPlayer && maxTankedPlayer > 0
                        ? (p.damageTaken / maxTankedPlayer) * 100
                        : p.tankedPct
                      : SETTINGS.history.general.state.relativeToTopDPSPlayer &&
                          maxDpsPlayer > 0
                        ? (p.totalDmg / maxDpsPlayer) * 100
                        : p.dmgPct}
                />
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else if charId && skillsWindow && selectedPlayer}
    <!-- Player Skills View -->
    <div class="mb-4">
      <div class="flex items-center gap-3 mb-2">
        <button
          onclick={backToEncounter}
          class="p-1.5 text-neutral-400 hover:text-neutral-200 transition-colors rounded hover:bg-neutral-800"
          aria-label="Back to encounter"
        >
          <svg
            class="w-5 h-5"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 19l-7-7 7-7"
            />
          </svg>
        </button>
        <div>
          <h2 class="text-xl font-semibold text-foreground">Skill Breakdown</h2>
          <div class="text-sm text-neutral-400">
            Player: {getDisplayName({
              player: {
                uid: selectedPlayer.uid,
                name: selectedPlayer.name,
                className: selectedPlayer.className,
                classSpecName: selectedPlayer.classSpecName,
              },
              showYourNameSetting: settings.state.history.general.showYourName,
              showOthersNameSetting:
                settings.state.history.general.showOthersName,
              isLocalPlayer: selectedPlayer.isLocalPlayer,
            })} <span class="text-neutral-500">#{selectedPlayer.uid}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="overflow-x-auto rounded border border-border/60 bg-card/30">
      <table class="w-full border-collapse">
        <thead>
          <tr class="bg-popover/60">
            <th
              class="px-3 py-3 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground"
              >Skill</th
            >
            {#each visibleSkillColumns as col (col.key)}
              <th
                class="px-3 py-3 text-right text-xs font-medium uppercase tracking-wider text-muted-foreground"
                >{col.header}</th
              >
            {/each}
          </tr>
        </thead>
        <tbody class="bg-background/40">
          {#each skillsWindow.skillRows as s (s.name)}
            <tr
              class="relative border-t border-border/40 hover:bg-muted/60 transition-colors"
            >
              <td class="px-3 py-3 text-sm text-muted-foreground relative z-10"
                >{s.name}</td
              >
              {#each visibleSkillColumns as col (col.key)}
                <td
                  class="px-3 py-3 text-right text-sm text-muted-foreground relative z-10"
                >
                  {#if (col.key === "totalDmg" || col.key === "dps") && (skillType === "tanked" ? SETTINGS.history.general.state.shortenTps : SETTINGS.history.general.state.shortenDps)}
                    <AbbreviatedNumber num={s[col.key] ?? 0} />
                  {:else}
                    {col.format(s[col.key] ?? 0)}
                  {/if}
                </td>
              {/each}
              <TableRowGlow
                className={selectedPlayer.className}
                percentage={skillType === "heal"
                  ? SETTINGS.history.general.state.relativeToTopHealSkill &&
                    maxHealSkill > 0
                    ? (s.totalDmg / maxHealSkill) * 100
                    : s.dmgPct
                  : skillType === "tanked"
                    ? SETTINGS.history.general.state.relativeToTopTankedSkill &&
                      maxTankedSkill > 0
                      ? (s.totalDmg / maxTankedSkill) * 100
                      : s.dmgPct
                    : SETTINGS.history.general.state.relativeToTopDPSSkill &&
                        maxDpsSkill > 0
                      ? (s.totalDmg / maxDpsSkill) * 100
                      : s.dmgPct}
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

<!-- Delete Confirmation Modal -->
{#if showDeleteModal}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center"
    role="dialog"
    aria-modal="true"
    aria-labelledby="delete-modal-title"
  >
    <!-- Backdrop -->
    <button
      class="absolute inset-0 bg-black/60 backdrop-blur-sm"
      onclick={closeDeleteModal}
      aria-label="Close modal"
    ></button>

    <!-- Modal Content -->
    <div
      class="relative bg-card border border-border rounded-lg shadow-xl max-w-md w-full mx-4 p-6"
    >
      <div class="flex items-start gap-4">
        <!-- Warning Icon -->
        <div
          class="flex-shrink-0 w-10 h-10 rounded-full bg-destructive/10 flex items-center justify-center"
        >
          <svg
            class="w-5 h-5 text-destructive"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
        </div>

        <div class="flex-1">
          <h3
            id="delete-modal-title"
            class="text-lg font-semibold text-foreground"
          >
            Delete Encounter
          </h3>
          <p class="mt-2 text-sm text-muted-foreground">
            Are you sure you want to delete this encounter? This action cannot
            be undone and all associated data will be permanently removed.
          </p>
        </div>
      </div>

      <!-- Actions -->
      <div class="mt-6 flex justify-end gap-3">
        <button
          onclick={closeDeleteModal}
          disabled={isDeleting}
          class="px-4 py-2 text-sm rounded-md border border-border bg-popover text-foreground hover:bg-muted/40 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Cancel
        </button>
        <button
          onclick={confirmDeleteEncounter}
          disabled={isDeleting}
          class="px-4 py-2 text-sm rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center gap-2"
        >
          {#if isDeleting}
            <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              ></circle>
              <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            Deleting...
          {:else}
            Delete
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

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
