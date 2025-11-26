<script lang="ts">
  /**
   * @file This component displays the header of the live meter,
   * which includes encounter statistics, and buttons for controlling the meter.
   */
  import {
    getCurrentWebviewWindow,
    WebviewWindow,
  } from "@tauri-apps/api/webviewWindow";

  // import CameraIcon from "virtual:icons/lucide/camera";
  import BossHealth from "./boss-health.svelte";
  import PauseIcon from "virtual:icons/lucide/pause";
  import PlayIcon from "virtual:icons/lucide/play";
  import MinusIcon from "virtual:icons/lucide/minus";
  import SettingsIcon from "virtual:icons/lucide/settings";
  import RefreshCwIcon from "virtual:icons/lucide/refresh-cw";
  import CrownIcon from "virtual:icons/lucide/crown";

  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getVersion } from "@tauri-apps/api/app";
  import {
    onEncounterUpdate,
    onResetEncounter,
    resetEncounter,
    togglePauseEncounter,
    setBossOnlyDps,
    type HeaderInfo,
  } from "$lib/api";
  // import { takeScreenshot, tooltip } from "$lib/utils.svelte";
  import { tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import { emitTo } from "@tauri-apps/api/event";
  import { SETTINGS } from "$lib/settings-store";
  import { getLiveDungeonLog } from "$lib/stores/live-meter-store.svelte";

  let fightStartTimestampMs = $state(0);
  let clientElapsedMs = $state(0);
  let animationFrameId: number | null = null;

  // Reactive dungeon log state with derived active segment info
  // also does a fake reset on segment type change for the live meter
  let dungeonLog = $derived(getLiveDungeonLog());
  let activeSegment = $derived(
    dungeonLog?.segments?.find((s) => !s.endedAtMs) ?? null,
  );
  let activeSegmentInfo = $derived.by(() => {
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

  // Client-side timer loop
  function updateClientTimer() {
    if (fightStartTimestampMs > 0 && !isEncounterPaused) {
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

      // Sync fight start timestamp from backend (acts as sync point)
      if (
        newFightStartTimestamp > 0 &&
        newFightStartTimestamp !== fightStartTimestampMs
      ) {
        fightStartTimestampMs = newFightStartTimestamp;
        clientElapsedMs = Date.now() - fightStartTimestampMs;
      }

      // Update other header info
      headerInfo = newHeaderInfo;
      isEncounterPaused = event.payload.isPaused;

      // Reset client timer if encounter is reset (fightStartTimestampMs becomes 0)
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

    // Listen for reset-encounter event (fired on server change)
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

    // Start the client-side timer loop
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
  // Use live.general for bossOnlyDps
  let bossOnlyDpsEnabled = $derived(SETTINGS.live.general.state.bossOnlyDps);

  // const {
  //   screenshotDiv,
  // }: {
  //   screenshotDiv?: HTMLElement;
  // } = $props();
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

  function toggleBossOnlyDamage() {
    const nextValue = !SETTINGS.live.general.state.bossOnlyDps;
    SETTINGS.live.general.state.bossOnlyDps = nextValue;
    void setBossOnlyDps(nextValue);
  }

  // When reset encounter button is pressed -> reset boss hp bar info
  function handleResetEncounter() {
    resetTimer();
    isEncounterPaused = false;
    void resetEncounter();
  }
</script>

<!-- 2x2 Grid Header Layout -->
<header
  data-tauri-drag-region
  class="grid w-full grid-cols-[1fr_auto] grid-rows-[auto_auto] pb-2 text-sm"
>
  <!-- Row 1, Col 1: Version + Timer -->
  <div
    class="col-start-1 row-start-1 flex items-center overflow-hidden gap-4 min-w-0"
    data-tauri-drag-region
  >
    <div class="flex items-center gap-2 shrink-0">
      <span
        class="text-sm font-medium text-muted-foreground uppercase tracking-wider leading-none"
        >Timer</span
      >
      <span
        class="text-lg font-bold text-foreground tabular-nums tracking-tight leading-none"
        {@attach tooltip(() => "Time Elapsed")}
        >{formatElapsed(clientElapsedMs)}</span
      >
    </div>
    {#if headerInfo.sceneName}
      <span
        class="text-base text-muted-foreground font-medium shrink-0 leading-none"
        {@attach tooltip(() => headerInfo.sceneName || "")}
        >{headerInfo.sceneName}</span
      >
    {/if}
    {#if activeSegmentInfo}
      <span
        class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded border shrink-0 {activeSegmentInfo.type ===
        'boss'
          ? 'border-orange-500/30 bg-orange-500/10 text-orange-400'
          : 'border-slate-500/30 bg-slate-500/10 text-slate-400'} text-xs"
      >
        <span class="font-semibold tracking-wide"
          >{activeSegmentInfo.label}</span
        >
        <span class="text-muted-foreground">•</span>
        <span>{Math.floor(activeSegmentInfo.durationSecs)}s</span>
        <!-- totalDamage removed per request: do not display segment total damage in live meter -->
      </span>
    {/if}
  </div>

  <!-- Row 1, Col 2: Buttons -->
  <div
    class="col-start-2 row-start-1 flex items-center justify-self-end gap-2 shrink-0"
  >
    <button
      class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg p-2 transition-all duration-200"
      onclick={handleResetEncounter}
      {@attach tooltip(() => "Reset Encounter")}
    >
      <RefreshCwIcon class="size-5" />
    </button>
    <button
      class="{isEncounterPaused
        ? 'text-[oklch(0.65_0.1_145)] bg-[oklch(0.9_0.02_145)]/30'
        : 'text-muted-foreground'} hover:text-foreground hover:bg-popover/60 rounded-lg p-2 transition-all duration-200"
      onclick={() => {
        togglePauseEncounter();
        isEncounterPaused = !isEncounterPaused;
      }}
    >
      {#if isEncounterPaused}
        <PlayIcon {@attach tooltip(() => "Resume Encounter")} class="size-5" />
      {:else}
        <PauseIcon {@attach tooltip(() => "Pause Encounter")} class="size-5" />
      {/if}
    </button>
    <button
      class="rounded-lg p-2 transition-all duration-200 {bossOnlyDpsEnabled
        ? 'text-[oklch(0.75_0.1_95)] bg-[oklch(0.95_0.02_95)]/30 hover:bg-[oklch(0.95_0.02_95)]/50'
        : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
      class:boss-only-active={bossOnlyDpsEnabled}
      aria-pressed={bossOnlyDpsEnabled}
      onclick={toggleBossOnlyDamage}
      {@attach tooltip(() =>
        bossOnlyDpsEnabled
          ? "Boss Only Damage Enabled"
          : "Enable Boss Only Damage",
      )}
    >
      <CrownIcon class="size-5" />
    </button>

    <div class="h-5 w-px bg-neutral-700/60"></div>
    <!-- <button
      class="text-neutral-400 hover:text-neutral-100 hover:bg-neutral-800/60 rounded-lg p-1.5 transition-all duration-200"
      onclick={() => appWindow.setIgnoreCursorEvents(true)}
      {@attach tooltip(() => 'Clickthrough')}
    >
      <PointerIcon class="size-5" />
    </button> -->
    <button
      class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg p-2 transition-all duration-200"
      onclick={() => openSettings()}
      {@attach tooltip(() => "Settings")}
    >
      <SettingsIcon class="size-5" />
    </button>
    <button
      class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg p-2 transition-all duration-200"
      onclick={() => appWindow.hide()}
      {@attach tooltip(() => "Minimize")}
    >
      <MinusIcon class="size-5" />
    </button>
  </div>

  <!-- Row 2, Col 1: Stats summary + Boss Health -->
  <div
    class="col-start-1 row-start-2 flex overflow-hidden items-center gap-5 min-w-0"
  >
    <!-- Stats -->
    <div class="flex overflow-hidden items-center gap-5">
      <div class="flex items-center gap-2 shrink-0">
        <span
          class="text-base font-bold text-muted-foreground uppercase tracking-wider"
          {@attach tooltip(() => "Total Damage Dealt")}>T.DMG</span
        >
        <span
          class="text-lg font-bold text-foreground"
          {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}
          ><AbbreviatedNumber num={Number(headerInfo.totalDmg)} /></span
        >
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <span
          class="text-base font-bold text-muted-foreground uppercase tracking-wider"
          {@attach tooltip(() => "Total Damage per Second")}>T.DPS</span
        >
        <span
          class="text-lg font-bold text-foreground"
          {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}
          ><AbbreviatedNumber num={headerInfo.totalDps} /></span
        >
      </div>
    </div>

    <!-- Boss Health -->
    <div class="flex items-center gap-2 shrink-0">
      <span
        class="text-base font-bold text-muted-foreground uppercase tracking-wider"
        {@attach tooltip(() => "Total Damage per Second")}>BOSS</span
      >
      <BossHealth />
    </div>
  </div>

  <!-- Row 2, Col 2: DPS/HEAL/TANKED Tabs (Connected) -->
  <div
    class="col-start-2 row-start-2 justify-self-end flex items-center border border-border rounded-lg overflow-hidden bg-popover/30 shrink-0 m-1"
  >
    <button
      class={`px-3.5 py-1.5 transition-all duration-200 font-bold tracking-wider uppercase text-[11px] border-r border-border whitespace-nowrap ${$page.url.pathname.includes("dps") ? "bg-muted text-foreground" : "text-muted-foreground hover:text-foreground hover:bg-popover/60"}`}
      aria-current={$page.url.pathname.includes("dps") ? "page" : undefined}
      onclick={() => goto(resolve("/live/dps"))}>DPS</button
    >
    <button
      class={`px-3.5 py-1.5 transition-all duration-200 font-bold tracking-wider uppercase text-[11px] border-r border-border whitespace-nowrap ${$page.url.pathname.includes("heal") ? "bg-muted text-foreground" : "text-muted-foreground hover:text-foreground hover:bg-popover/60"}`}
      aria-current={$page.url.pathname.includes("heal") ? "page" : undefined}
      onclick={() => goto(resolve("/live/heal"))}>HEAL</button
    >
    <button
      class={`px-3.5 py-1.5 transition-all duration-200 font-bold tracking-wider uppercase text-[11px] whitespace-nowrap ${$page.url.pathname.includes("tanked") ? "bg-muted text-foreground" : "text-muted-foreground hover:text-foreground hover:bg-popover/60"}`}
      aria-current={$page.url.pathname.includes("tanked") ? "page" : undefined}
      onclick={() => goto(resolve("/live/tanked"))}>TANKED</button
    >
  </div>
</header>
