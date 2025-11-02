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
      bosses: [],
    };
  }

  onMount(() => {
    let encounterUnlisten: (() => void) | null = null;
    let resetUnlisten: (() => void) | null = null;

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
    bosses: [],
  });
  let isEncounterPaused = $state(false);
  let bossOnlyDpsEnabled = $derived(SETTINGS.general.state.bossOnlyDps);
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
    const nextValue = !SETTINGS.general.state.bossOnlyDps;
    SETTINGS.general.state.bossOnlyDps = nextValue;
    void setBossOnlyDps(nextValue);
  }
    // When reset encounter button is pressed -> reset boss hp bar info
  function handleResetEncounter() {
    resetTimer();
    isEncounterPaused = false;
    void resetEncounter();
  }
</script>

<!-- justify-between to create left/right sides -->
<header data-tauri-drag-region class="flex w-full items-center justify-between gap-1 bg-neutral-900 px-4 py-3 text-sm rounded-t-lg">
  <!-- Left side -->
  <div class="flex flex-col" data-tauri-drag-region>
    <div class="flex flex-wrap items-center gap-2" data-tauri-drag-region>
      <span class="text-sm font-medium" {@attach tooltip(() => "Time Elapsed")}>{formatElapsed(clientElapsedMs)}</span>
      <span class="text-sm"><span {@attach tooltip(() => "Total Damage Dealt")}>T.DMG</span> <span {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}><AbbreviatedNumber num={Number(headerInfo.totalDmg)} /></span></span>
      <span class="text-sm"><span {@attach tooltip(() => "Total Damage per Second")}>T.DPS</span> <span {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}><AbbreviatedNumber num={headerInfo.totalDps} /></span></span>
    </div>

    {#if headerInfo.bosses.length > 0}
      <div class="mt-1 flex flex-col gap-0.5" data-tauri-drag-region>
        {#each headerInfo.bosses as boss (boss.uid)}
          <div class="flex items-center gap-2 text-xs text-neutral-300" data-tauri-drag-region>
            <span class="w-28 truncate" {@attach tooltip(() => boss.name)}>{boss.name}</span>
            <div class="relative h-2 w-40 rounded bg-neutral-800 pointer-events-none">
              <div
                class="absolute inset-y-0 left-0 rounded bg-amber-500 pointer-events-none"
                style={`width: ${boss.maxHp && boss.currentHp !== null ? Math.min(100, Math.max(0, (boss.currentHp / boss.maxHp) * 100)) : 0}%`}
              />
            </div>
            <span>{boss.currentHp !== null ? boss.currentHp.toLocaleString() : "?"}{boss.maxHp ? ` / ${boss.maxHp.toLocaleString()}` : ""}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
  <!-- Right side -->
  <span class="flex gap-2">
    <!-- TODO: add responsive clicks, toaster -->
    <!-- <button
      onclick={async () => takeScreenshot(screenshotDiv)}
      {@attach tooltip(() => "Screenshot to Clipboard")}
    >
      <CameraIcon />
    </button> -->
    <button onclick={handleResetEncounter} {@attach tooltip(() => "Reset Encounter")}><RefreshCwIcon class="size-5" /></button>
    <button
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
      class="boss-only-toggle"
      class:boss-only-active={bossOnlyDpsEnabled}
      aria-pressed={bossOnlyDpsEnabled}
      onclick={toggleBossOnlyDamage}
      {@attach tooltip(() => (bossOnlyDpsEnabled ? "Boss Only Damage Enabled" : "Enable Boss Only Damage"))}
    >
      <CrownIcon class="size-5" />
    </button>
    <button onclick={() => appWindow.setIgnoreCursorEvents(true)} {@attach tooltip(() => "Clickthrough")}><PointerIcon class="size-5" /></button>
    <button onclick={() => openSettings()} {@attach tooltip(() => "Settings")}><SettingsIcon class="size-5" /></button>
    <button onclick={() => appWindow.hide()} {@attach tooltip(() => "Minimize")}><MinusIcon class="size-5" /></button>
  </span>
</header>

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
