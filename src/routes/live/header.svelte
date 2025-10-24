<script lang="ts">
  import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow";

  // import CameraIcon from "virtual:icons/lucide/camera";
  import TimerResetIcon from "virtual:icons/lucide/timer-reset";
  import PauseIcon from "virtual:icons/lucide/pause";
  import PlayIcon from "virtual:icons/lucide/play";
  import MinusIcon from "virtual:icons/lucide/minus";
  import PointerIcon from "virtual:icons/lucide/pointer";
  import SettingsIcon from "virtual:icons/lucide/settings";
  import RefreshCwIcon from "virtual:icons/lucide/refresh-cw";
  import CrownIcon from "virtual:icons/lucide/crown";

  import { onMount, tick } from "svelte";
  import { onEncounterUpdate, onResetEncounter, resetEncounter, togglePauseEncounter, setBossOnlyDps, type HeaderInfo } from "$lib/api";
  // import { takeScreenshot, tooltip } from "$lib/utils.svelte";
  import { tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import { emitTo } from "@tauri-apps/api/event";
  import { SETTINGS } from "$lib/settings-store";

  let fightStartTimestampMs = $state(0);
  let clientElapsedMs = $state(0);
  let animationFrameId: number | null = null;

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
    };
  }

  onMount(() => {
    let encounterUnlisten: (() => void) | null = null;
    let resetUnlisten: (() => void) | null = null;

    // Sync initial boss-only setting when header mounts
    setBossOnlyDps(SETTINGS.general.state.bossOnlyDps).catch(() => {});

    onEncounterUpdate((event) => {
      const newHeaderInfo = event.payload.headerInfo;
      const newFightStartTimestamp = newHeaderInfo.fightStartTimestampMs;

      // Sync fight start timestamp from backend (acts as sync point)
      if (newFightStartTimestamp > 0 && newFightStartTimestamp !== fightStartTimestampMs) {
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
      encounterUnlisten = fn;
    });

    // Listen for reset-encounter event (fired on server change)
    onResetEncounter(() => {
      resetTimer();
    }).then((fn) => {
      resetUnlisten = fn;
    });

    // Start the client-side timer loop
    animationFrameId = requestAnimationFrame(updateClientTimer);

    return () => {
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
  });
  let isEncounterPaused = $state(false);
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

  function toggleBossOnly() {
    const next = !SETTINGS.general.state.bossOnlyDps;
    SETTINGS.general.state.bossOnlyDps = next;
    setBossOnlyDps(next);
  }
</script>

<!-- justify-between to create left/right sides -->
<header data-tauri-drag-region class="sticky top-0 flex h-7 w-full items-center justify-between bg-neutral-900/80 px-1">
  <!-- Left side -->
  <span>
    <span {@attach tooltip(() => "Time Elapsed")}>{formatElapsed(clientElapsedMs)}</span>
    <span><span {@attach tooltip(() => "Total Damage Dealt")}>T.DMG</span> <span {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}><AbbreviatedNumber num={Number(headerInfo.totalDmg)} /></span></span>
    <span><span {@attach tooltip(() => "Total Damage per Second")}>T.DPS</span> <span {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}><AbbreviatedNumber num={headerInfo.totalDps} /></span></span>
  </span>
  <!-- Right side -->
  <span class="flex gap-1">
    <!-- Boss-only toggle -->
    <button
      class={SETTINGS.general.state.bossOnlyDps ? "text-yellow-400" : "text-neutral-300"}
      onclick={toggleBossOnly}
      {@attach tooltip(() => SETTINGS.general.state.bossOnlyDps ? "Boss-only damage: ON" : "Boss-only damage: OFF")}
    >
      <CrownIcon />
    </button>
    <!-- TODO: add responsive clicks, toaster -->
    <!-- <button
      onclick={async () => takeScreenshot(screenshotDiv)}
      {@attach tooltip(() => "Screenshot to Clipboard")}
    >
      <CameraIcon />
    </button> -->
        <button onclick={() => resetEncounter()} {@attach tooltip(() => "Reset Encounter")}><RefreshCwIcon /></button>
        <button
          onclick={() => {
            togglePauseEncounter();
            isEncounterPaused = !isEncounterPaused;
          }}
        >
          {#if isEncounterPaused}
            <PlayIcon {@attach tooltip(() => "Resume Encounter")} />
          {:else}
            <PauseIcon {@attach tooltip(() => "Pause Encounter")} />
          {/if}
        </button>
    <button onclick={() => appWindow.setIgnoreCursorEvents(true)} {@attach tooltip(() => "Clickthrough")}><PointerIcon /></button>
    <button onclick={() => openSettings()} {@attach tooltip(() => "Settings")}><SettingsIcon /></button>
    <button onclick={() => appWindow.hide()} {@attach tooltip(() => "Minimize")}><MinusIcon /></button>
  </span>
</header>
