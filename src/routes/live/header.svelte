<script lang="ts">
  /**
   * @file This component displays the header of the live meter,
   * which includes encounter statistics, and buttons for controlling the meter.
   */
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
  import MinimizeIcon from "virtual:icons/lucide/minimize-2";

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
      sceneId: null,
      sceneName: null,
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
    sceneId: null,
    sceneName: null,
  });
  let isEncounterPaused = $state(false);
  let bossOnlyDpsEnabled = $derived(SETTINGS.general.state.bossOnlyDps);
  let density = $derived(SETTINGS.accessibility.state.density ?? "comfortable");
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

  let densityAnimating = $state(false);

  function triggerDensityAnimation(next: string) {
    // Shrink while in medium/compact, restore when going back to comfortable
    densityAnimating = next !== "comfortable";
    if (next === "comfortable") {
      // Allow one frame for CSS transition to play back to normal
      setTimeout(() => {
        densityAnimating = false;
      }, 150);
    }
  }

  function toggleDensity() {
    const current = SETTINGS.accessibility.state.density ?? "comfortable";
    const next = current === "comfortable" ? "medium" : current === "medium" ? "compact" : "comfortable";
    SETTINGS.accessibility.state.density = next;
    triggerDensityAnimation(next);
  }
    // When reset encounter button is pressed -> reset boss hp bar info
  function handleResetEncounter() {
    resetTimer();
    isEncounterPaused = false;
    void resetEncounter();
  }
</script>

<!-- justify-between to create left/right sides -->
<header data-tauri-drag-region class="flex w-full items-center justify-between gap-1 bg-neutral-900 {density === 'comfortable' ? 'px-4 py-3 text-sm' : density === 'medium' ? 'px-3 py-2 text-[12px]' : 'px-2 py-1.5 text-[11px]'} rounded-t-lg">
  <!-- Left side -->
  <div class="flex flex-col" data-tauri-drag-region>
    <div class="flex flex-wrap items-center {density === 'comfortable' ? 'gap-2' : density === 'medium' ? 'gap-1.5' : 'gap-1.5'}" data-tauri-drag-region>
      {#if headerInfo.sceneName}
        <span class="{density === 'comfortable' ? 'text-sm' : density === 'medium' ? 'text-[12px]' : 'text-[11px]'} font-bold text-neutral-100" {@attach tooltip(() => headerInfo.sceneName || "")}>{headerInfo.sceneName}</span>
      {/if}
      <span class="{density === 'comfortable' ? 'text-sm' : density === 'medium' ? 'text-[12px]' : 'text-[11px]'} font-medium text-neutral-300" {@attach tooltip(() => "Time Elapsed")}>{formatElapsed(clientElapsedMs)}</span>
      <span class="{density === 'comfortable' ? 'text-sm' : density === 'medium' ? 'text-[12px]' : 'text-[11px]'}"><span {@attach tooltip(() => "Total Damage Dealt")}>T.DMG</span> <span {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}><AbbreviatedNumber num={Number(headerInfo.totalDmg)} /></span></span>
      <span class="{density === 'comfortable' ? 'text-sm' : density === 'medium' ? 'text-[12px]' : 'text-[11px]'}"><span {@attach tooltip(() => "Total Damage per Second")}>T.DPS</span> <span {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}><AbbreviatedNumber num={headerInfo.totalDps} /></span></span>
    </div>
  </div>
  <!-- Right side -->
  <span class="flex {density === 'comfortable' ? 'gap-2' : 'gap-1.5'}">
    <!-- TODO: add responsive clicks, toaster -->
    <!-- <button
      onclick={async () => takeScreenshot(screenshotDiv)}
      {@attach tooltip(() => "Screenshot to Clipboard")}
    >
      <CameraIcon />
    </button> -->
    <button onclick={handleResetEncounter} {@attach tooltip(() => "Reset Encounter")}><RefreshCwIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} /></button>
    <button
      onclick={() => {
        togglePauseEncounter();
        isEncounterPaused = !isEncounterPaused;
      }}
    >
      {#if isEncounterPaused}
        <PlayIcon {@attach tooltip(() => "Resume Encounter")} class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} />
      {:else}
        <PauseIcon {@attach tooltip(() => "Pause Encounter")} class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} />
      {/if}
    </button>
    <button
      class="boss-only-toggle"
      class:boss-only-active={bossOnlyDpsEnabled}
      aria-pressed={bossOnlyDpsEnabled}
      onclick={toggleBossOnlyDamage}
      {@attach tooltip(() => (bossOnlyDpsEnabled ? "Boss Only Damage Enabled" : "Enable Boss Only Damage"))}
    >
      <CrownIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4.5' : 'size-4'} />
    </button>
    <button
      class="compact-mode-toggle"
      class:compact-mode-active={density !== 'comfortable'}
      class:density-animating={densityAnimating}
      aria-pressed={density !== 'comfortable'}
      onclick={toggleDensity}
      {@attach tooltip(() => density === 'comfortable' ? "Density: Comfortable" : density === 'medium' ? "Density: Medium" : "Density: Compact")}>
      <MinimizeIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4.5' : 'size-4'} />
    </button>
    <button onclick={() => appWindow.setIgnoreCursorEvents(true)} {@attach tooltip(() => "Clickthrough")}><PointerIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} /></button>
    <button onclick={() => openSettings()} {@attach tooltip(() => "Settings")}><SettingsIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} /></button>
    <button onclick={() => appWindow.hide()} {@attach tooltip(() => "Minimize")}><MinusIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} /></button>
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

  .compact-mode-toggle {
    transition: color 150ms ease, transform 150ms ease;
    transform-origin: center;
  }

  .compact-mode-toggle:hover {
    color: #60a5fa;
  }

  .compact-mode-toggle.compact-mode-active {
    color: #60a5fa;
  }

  .compact-mode-toggle.density-animating {
    transform: scale(0.8);
  }
</style>
