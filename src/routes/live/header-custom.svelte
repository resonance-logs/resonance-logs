<script lang="ts">
  /**
   * @file Fully customizable header component for the live meter.
   * Renders header elements based on user settings.
   */
  import {
    getCurrentWebviewWindow,
    WebviewWindow,
  } from "@tauri-apps/api/webviewWindow";

  import PauseIcon from "virtual:icons/lucide/pause";
  import PlayIcon from "virtual:icons/lucide/play";
  import MinusIcon from "virtual:icons/lucide/minus";
  import SettingsIcon from "virtual:icons/lucide/settings";
  import RefreshCwIcon from "virtual:icons/lucide/refresh-cw";

  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import {
    onEncounterUpdate,
    onResetEncounter,
    resetEncounter,
    togglePauseEncounter,
    type HeaderInfo,
  } from "$lib/api";
  import { tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import { emitTo } from "@tauri-apps/api/event";
  import { SETTINGS } from "$lib/settings-store";
  import { getLiveDungeonLog } from "$lib/stores/live-meter-store.svelte";
  import { DUMMY_HEADER_INFO, DUMMY_SEGMENT_INFO } from "$lib/dummy-data";

  // Get header settings
  let h = $derived(SETTINGS.live.headerCustomization.state);

  // Check if dummy data mode is enabled
  let useDummyData = $derived(SETTINGS.live.general.state.useDummyData);

  let fightStartTimestampMs = $state(0);
  let clientElapsedMs = $state(0);
  let animationFrameId: number | null = null;

  // Reactive dungeon log state with derived active segment info
  let dungeonLog = $derived(getLiveDungeonLog());
  let activeSegment = $derived(
    dungeonLog?.segments?.find((s) => !s.endedAtMs) ?? null,
  );
  let activeSegmentInfo = $derived.by(() => {
    // Use dummy segment info when dummy data is enabled
    if (useDummyData) {
      return DUMMY_SEGMENT_INFO;
    }

    if (!activeSegment) return null;

    const durationSecs = Math.max(
      1,
      ((activeSegment.endedAtMs ?? Date.now()) - activeSegment.startedAtMs) /
        1000,
    );

    return {
      durationSecs,
      type: activeSegment.segmentType,
      label:
        activeSegment.segmentType === "boss"
          ? (activeSegment.bossName ?? "Boss Segment")
          : "Trash Segment",
    };
  });

  // Client-side timer loop - only runs when not using dummy data
  function updateClientTimer() {
    if (!useDummyData && fightStartTimestampMs > 0 && !isEncounterPaused) {
      clientElapsedMs = Date.now() - fightStartTimestampMs;
    }
    animationFrameId = requestAnimationFrame(updateClientTimer);
  }

  function resetTimer() {
    fightStartTimestampMs = 0;
    clientElapsedMs = 0;
    headerInfo = {
      totalDps: 0,
      totalDmg: 0,
      elapsedMs: 0,
      fightStartTimestampMs: 0,
      bosses: [],
      sceneId: null,
      sceneName: null,
      currentSegmentType: null,
      currentSegmentName: null,
    };
  }

  onMount(() => {
    let encounterUnlisten: (() => void) | null = null;
    let resetUnlisten: (() => void) | null = null;
    let isDestroyed = false;

    onEncounterUpdate((event) => {
      if (isDestroyed) return;
      const newHeaderInfo = event.payload.headerInfo;
      const newFightStartTimestamp = newHeaderInfo.fightStartTimestampMs;

      if (
        newFightStartTimestamp > 0 &&
        newFightStartTimestamp !== fightStartTimestampMs
      ) {
        fightStartTimestampMs = newFightStartTimestamp;
        clientElapsedMs = Date.now() - fightStartTimestampMs;
      }

      headerInfo = newHeaderInfo;
      isEncounterPaused = event.payload.isPaused;

      if (newFightStartTimestamp === 0) {
        fightStartTimestampMs = 0;
        clientElapsedMs = 0;
      }
    }).then((fn) => {
      if (isDestroyed) {
        fn();
      } else {
        encounterUnlisten = fn;
      }
    });

    onResetEncounter(() => {
      if (isDestroyed) return;
      resetTimer();
    }).then((fn) => {
      if (isDestroyed) {
        fn();
      } else {
        resetUnlisten = fn;
      }
    });

    animationFrameId = requestAnimationFrame(updateClientTimer);

    return () => {
      isDestroyed = true;
      if (encounterUnlisten) encounterUnlisten();
      if (resetUnlisten) resetUnlisten();
      if (animationFrameId !== null) {
        cancelAnimationFrame(animationFrameId);
      }
    };
  });

  function formatElapsed(msElapsed: number) {
    const totalSeconds = Math.floor(Number(msElapsed) / 1000);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }

  let headerInfo: HeaderInfo = $state({
    totalDps: 0,
    totalDmg: 0,
    elapsedMs: 0,
    fightStartTimestampMs: 0,
    bosses: [],
    sceneId: null,
    sceneName: null,
    currentSegmentType: null,
    currentSegmentName: null,
  });
  let isEncounterPaused = $state(false);

  // Display values - use dummy data when enabled
  let displayHeaderInfo = $derived(
    useDummyData ? DUMMY_HEADER_INFO : headerInfo,
  );
  let displayElapsedMs = $derived(
    useDummyData ? DUMMY_HEADER_INFO.elapsedMs : clientElapsedMs,
  );
  let displaySceneName = $derived(
    useDummyData ? DUMMY_HEADER_INFO.sceneName : headerInfo.sceneName,
  );
  let displayBosses = $derived(
    useDummyData ? DUMMY_HEADER_INFO.bosses : headerInfo.bosses,
  );

  const appWindow = getCurrentWebviewWindow();

  async function openSettings() {
    const mainWindow = await WebviewWindow.getByLabel("main");
    if (mainWindow !== null) {
      await mainWindow?.unminimize();
      await mainWindow?.show();
      await mainWindow?.setFocus();
      await emitTo("main", "navigate", "/main/settings");
    }
  }

  function handleResetEncounter() {
    resetTimer();
    isEncounterPaused = false;
    void resetEncounter();
  }

  // Check if we have any row 1 left content
  let hasRow1Left = $derived(
    h.showTimer || h.showSceneName || h.showSegmentInfo,
  );

  // Check if we have any row 1 right content (buttons)
  let hasRow1Right = $derived(
    h.showResetButton ||
      h.showPauseButton ||
      h.showSettingsButton ||
      h.showMinimizeButton,
  );

  // Check if we have any row 2 left content
  let hasRow2Left = $derived(
    h.showTotalDamage || h.showTotalDps || h.showBossHealth,
  );

  // Check if we have any row 2 content at all
  let hasRow2 = $derived(hasRow2Left || h.showNavigationTabs);

  // Check if we have any row 1 content at all
  let hasRow1 = $derived(hasRow1Left || hasRow1Right);
</script>

{#if hasRow1 || hasRow2}
  <header
    data-tauri-drag-region
    class="grid w-full grid-cols-[1fr_auto] text-sm"
    class:grid-rows-1={!hasRow2}
    class:grid-rows-2={hasRow2}
    style="padding: {h.headerPadding}px; padding-bottom: {h.headerPadding +
      4}px"
  >
    <!-- Row 1, Col 1: Timer + Scene + Segment -->
    {#if hasRow1Left}
      <div
        class="col-start-1 row-start-1 flex items-center overflow-hidden gap-4 min-w-0"
        data-tauri-drag-region
      >
        {#if h.showTimer}
          <div class="flex items-center gap-2 shrink-0">
            {#if h.timerLabelFontSize > 0}
              <span
                class="font-medium text-muted-foreground uppercase tracking-wider leading-none"
                style="font-size: {h.timerLabelFontSize}px">Timer</span
              >
            {/if}
            <span
              class="font-bold text-foreground tabular-nums tracking-tight leading-none"
              style="font-size: {h.timerFontSize}px"
              {@attach tooltip(() => "Time Elapsed")}
              >{formatElapsed(displayElapsedMs)}</span
            >
          </div>
        {/if}

        {#if h.showSceneName && displaySceneName}
          <span
            class="text-muted-foreground font-medium shrink-0 leading-none"
            style="font-size: {h.sceneNameFontSize}px"
            {@attach tooltip(() => displaySceneName || "")}
            >{displaySceneName}</span
          >
        {/if}

        {#if h.showSegmentInfo && activeSegmentInfo}
          <span
            class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded border shrink-0 {activeSegmentInfo.type ===
            'boss'
              ? 'border-orange-500/30 bg-orange-500/10 text-orange-400'
              : 'border-slate-500/30 bg-slate-500/10 text-slate-400'}"
            style="font-size: {h.segmentFontSize}px"
          >
            <span class="font-semibold tracking-wide"
              >{activeSegmentInfo.label}</span
            >
            <span class="text-muted-foreground">•</span>
            <span>{Math.floor(activeSegmentInfo.durationSecs)}s</span>
          </span>
        {/if}
      </div>
    {/if}

    <!-- Row 1, Col 2: Control Buttons -->
    {#if hasRow1Right}
      <div
        class="col-start-2 row-start-1 flex items-center justify-self-end gap-2 shrink-0"
      >
        {#if h.showResetButton}
          <button
            class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg transition-all duration-200"
            style="padding: {h.resetButtonPadding}px"
            onclick={handleResetEncounter}
            {@attach tooltip(() => "Reset Encounter")}
          >
            <RefreshCwIcon
              style="width: {h.resetButtonSize}px; height: {h.resetButtonSize}px"
            />
          </button>
        {/if}

        {#if h.showPauseButton}
          <button
            class="{isEncounterPaused
              ? 'text-[oklch(0.65_0.1_145)] bg-[oklch(0.9_0.02_145)]/30'
              : 'text-muted-foreground'} hover:text-foreground hover:bg-popover/60 rounded-lg transition-all duration-200"
            style="padding: {h.pauseButtonPadding}px"
            onclick={() => {
              togglePauseEncounter();
              isEncounterPaused = !isEncounterPaused;
            }}
          >
            {#if isEncounterPaused}
              <PlayIcon
                {@attach tooltip(() => "Resume Encounter")}
                style="width: {h.pauseButtonSize}px; height: {h.pauseButtonSize}px"
              />
            {:else}
              <PauseIcon
                {@attach tooltip(() => "Pause Encounter")}
                style="width: {h.pauseButtonSize}px; height: {h.pauseButtonSize}px"
              />
            {/if}
          </button>
        {/if}

        {#if h.showSettingsButton}
          <button
            class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg transition-all duration-200"
            style="padding: {h.settingsButtonPadding}px"
            onclick={() => openSettings()}
            {@attach tooltip(() => "Settings")}
          >
            <SettingsIcon
              style="width: {h.settingsButtonSize}px; height: {h.settingsButtonSize}px"
            />
          </button>
        {/if}

        {#if h.showMinimizeButton}
          <button
            class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg transition-all duration-200"
            style="padding: {h.minimizeButtonPadding}px"
            onclick={() => appWindow.hide()}
            {@attach tooltip(() => "Minimize")}
          >
            <MinusIcon
              style="width: {h.minimizeButtonSize}px; height: {h.minimizeButtonSize}px"
            />
          </button>
        {/if}
      </div>
    {/if}

    <!-- Row 2, Col 1: Stats summary + Boss Health -->
    {#if hasRow2Left}
      <div
        class="col-start-1 row-start-2 flex overflow-hidden items-center gap-5 min-w-0"
      >
        <div class="flex overflow-hidden items-center gap-5">
          {#if h.showTotalDamage}
            <div class="flex items-center gap-2 shrink-0">
              <span
                class="font-bold text-muted-foreground uppercase tracking-wider"
                style="font-size: {h.totalDamageLabelFontSize}px"
                {@attach tooltip(() => "Total Damage Dealt")}>T.DMG</span
              >
              <span
                class="font-bold text-foreground"
                style="font-size: {h.totalDamageValueFontSize}px"
                {@attach tooltip(() =>
                  displayHeaderInfo.totalDmg.toLocaleString(),
                )}
                ><AbbreviatedNumber
                  num={Number(displayHeaderInfo.totalDmg)}
                /></span
              >
            </div>
          {/if}

          {#if h.showTotalDps}
            <div class="flex items-center gap-2 shrink-0">
              <span
                class="font-bold text-muted-foreground uppercase tracking-wider"
                style="font-size: {h.totalDpsLabelFontSize}px"
                {@attach tooltip(() => "Total Damage per Second")}>T.DPS</span
              >
              <span
                class="font-bold text-foreground"
                style="font-size: {h.totalDpsValueFontSize}px"
                {@attach tooltip(() =>
                  displayHeaderInfo.totalDps.toLocaleString(),
                )}><AbbreviatedNumber num={displayHeaderInfo.totalDps} /></span
              >
            </div>
          {/if}
        </div>

        {#if h.showBossHealth}
          <div class="flex items-center gap-2 shrink-0">
            <span
              class="font-bold text-muted-foreground uppercase tracking-wider"
              style="font-size: {h.bossHealthLabelFontSize}px"
              {@attach tooltip(() => "Boss Health")}>BOSS</span
            >
            <!-- Inline Boss Health Display -->
            {#if displayBosses.length > 0}
              <div class="flex flex-col gap-1">
                {#each displayBosses as boss (boss.uid)}
                  {@const hpPercent =
                    boss.maxHp && boss.currentHp !== null
                      ? Math.min(
                          100,
                          Math.max(0, (boss.currentHp / boss.maxHp) * 100),
                        )
                      : 0}
                  <div class="flex items-center gap-1 whitespace-nowrap">
                    <span
                      class="truncate text-foreground font-semibold tracking-tight"
                      style="font-size: {h.bossHealthNameFontSize}px"
                      {@attach tooltip(() => boss.name)}>{boss.name} -</span
                    >
                    <span
                      class="tabular-nums font-semibold text-foreground"
                      style="font-size: {h.bossHealthValueFontSize}px"
                    >
                      <AbbreviatedNumber
                        num={boss.currentHp !== null ? boss.currentHp : 0}
                      />
                      {#if boss.maxHp}
                        <span> / <AbbreviatedNumber num={boss.maxHp} /></span>
                        <span
                          class="text-destructive ml-1"
                          style="font-size: {h.bossHealthPercentFontSize}px"
                          >({hpPercent.toFixed(1)}%)</span
                        >
                      {/if}
                    </span>
                  </div>
                {/each}
              </div>
            {:else}
              <span
                class="text-neutral-500 font-medium italic"
                style="font-size: {h.bossHealthNameFontSize}px">No Boss</span
              >
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Row 2, Col 2: DPS/HEAL/TANKED Tabs -->
    {#if h.showNavigationTabs}
      <div
        class="col-start-2 row-start-2 justify-self-end flex items-stretch border border-border rounded-lg overflow-hidden bg-popover/30 shrink-0"
      >
        <button
          class="transition-all duration-200 font-bold tracking-wider uppercase border-r border-border whitespace-nowrap h-full flex items-center {$page.url.pathname.includes(
            'dps',
          )
            ? 'bg-muted text-foreground'
            : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
          style="font-size: {h.navTabFontSize}px; padding: {h.navTabPaddingY}px {h.navTabPaddingX}px"
          aria-current={$page.url.pathname.includes("dps") ? "page" : undefined}
          onclick={() => goto(resolve("/live/dps"))}>DPS</button
        >
        <button
          class="transition-all duration-200 font-bold tracking-wider uppercase border-r border-border whitespace-nowrap h-full flex items-center {$page.url.pathname.includes(
            'heal',
          )
            ? 'bg-muted text-foreground'
            : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
          style="font-size: {h.navTabFontSize}px; padding: {h.navTabPaddingY}px {h.navTabPaddingX}px"
          aria-current={$page.url.pathname.includes("heal")
            ? "page"
            : undefined}
          onclick={() => goto(resolve("/live/heal"))}>HEAL</button
        >
        <button
          class="transition-all duration-200 font-bold tracking-wider uppercase border-r border-border whitespace-nowrap h-full flex items-center {$page.url.pathname.includes(
            'tanked',
          )
            ? 'bg-muted text-foreground'
            : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
          style="font-size: {h.navTabFontSize}px; padding: {h.navTabPaddingY}px {h.navTabPaddingX}px"
          aria-current={$page.url.pathname.includes("tanked")
            ? "page"
            : undefined}
          onclick={() => goto(resolve("/live/tanked"))}>TANKED</button
        >
        <button
          class="transition-all duration-200 font-bold tracking-wider uppercase whitespace-nowrap h-full flex items-center {$page.url.pathname.includes(
            'buffs',
          )
            ? 'bg-muted text-foreground'
            : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
          style="font-size: {h.navTabFontSize}px; padding: {h.navTabPaddingY}px {h.navTabPaddingX}px"
          aria-current={$page.url.pathname.includes("buffs")
            ? "page"
            : undefined}
          onclick={() => goto(resolve("/live/buffs"))}>BUFFS</button
        >
      </div>
    {/if}
  </header>
{/if}
