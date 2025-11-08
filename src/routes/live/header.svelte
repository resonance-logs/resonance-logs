<script lang="ts">
  /**
   * @file This component displays the header of the live meter,
   * which includes encounter statistics, and buttons for controlling the meter.
   */
  import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow";

  // import CameraIcon from "virtual:icons/lucide/camera";
  import BossHealth from "./boss-health.svelte";
  import PauseIcon from "virtual:icons/lucide/pause";
  import PlayIcon from "virtual:icons/lucide/play";
  import MinusIcon from "virtual:icons/lucide/minus";
  import PointerIcon from "virtual:icons/lucide/pointer";
  import SettingsIcon from "virtual:icons/lucide/settings";
  import RefreshCwIcon from "virtual:icons/lucide/refresh-cw";
  import CrownIcon from "virtual:icons/lucide/crown";
  import MinimizeIcon from "virtual:icons/lucide/minimize-2";

  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getVersion } from "@tauri-apps/api/app";
  import { onEncounterUpdate, onResetEncounter, resetEncounter, togglePauseEncounter, setBossOnlyDps, type HeaderInfo } from "$lib/api";
  // import { takeScreenshot, tooltip } from "$lib/utils.svelte";
  import { tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import { emitTo } from "@tauri-apps/api/event";
  import { SETTINGS } from "$lib/settings-store";
  import { getDpsPlayers, getHealPlayers, getTankedPlayers } from "$lib/stores/live-meter-store.svelte";

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

  // Derived: number of players for current tab (DPS/HEAL/TANKED)
  let playersCount = $derived.by(() => {
    const path = $page.url.pathname;
    if (path.includes("/live/dps")) return getDpsPlayers().playerRows.length;
    if (path.includes("/live/heal")) return getHealPlayers().playerRows.length;
    if (path.includes("/live/tanked")) return getTankedPlayers().playerRows.length;
    // default to DPS if landing page
    return getDpsPlayers().playerRows.length || 0;
  });
</script>

<!-- Structured 3-row header layout with right-side Boss Health spanning rows 2-3 -->
<header
  data-tauri-drag-region
  class="grid w-full rounded-lg bg-neutral-900 backdrop-blur-sm grid-cols-[1fr_auto] grid-rows-[auto_auto_auto] gap-x-6 gap-y-1 items-center px-4 py-3 border border-neutral-800/70 shadow-sm
  {density === 'comfortable' ? 'text-[13px]' : density === 'medium' ? 'text-[12px]' : 'text-[11px]'}"
>
  <!-- Row 1, Col 1: Title -->
  <div class="col-start-1 row-start-1 flex items-center gap-2" data-tauri-drag-region>
    <span class="font-semibold text-neutral-100 tracking-tight">Live Meter</span>
    {#if headerInfo.sceneName}
      <span class="text-neutral-400/80 italic" {@attach tooltip(() => headerInfo.sceneName || "")}>({headerInfo.sceneName})</span>
    {/if}
  </div>

  <!-- Row 1, Col 2: Tabs + Buttons -->
  <div class="col-start-2 row-start-1 justify-self-end flex items-center {density === 'comfortable' ? 'gap-3' : 'gap-2'}">
    <!-- Tabs -->
  <span class="flex items-center gap-1.5">
      <button
        class={`rounded-md ${density === 'comfortable' ? 'px-3 py-1.5' : density === 'medium' ? 'px-2.5 py-1' : 'px-2 py-0.5'} transition-colors font-medium tracking-wide uppercase text-[11px] shadow-sm border ${$page.url.pathname.includes('dps') ? 'bg-neutral-800/90 text-neutral-100 border-neutral-700' : 'text-neutral-400 border-neutral-800 hover:text-neutral-200 hover:bg-neutral-800/60'}`}
        aria-current={$page.url.pathname.includes('dps') ? 'page' : undefined}
        onclick={() => goto(resolve('/live/dps'))}
      >DPS</button>
      <button
        class={`rounded-md ${density === 'comfortable' ? 'px-3 py-1.5' : density === 'medium' ? 'px-2.5 py-1' : 'px-2 py-0.5'} transition-colors font-medium tracking-wide uppercase text-[11px] shadow-sm border ${$page.url.pathname.includes('heal') ? 'bg-neutral-800/90 text-neutral-100 border-neutral-700' : 'text-neutral-400 border-neutral-800 hover:text-neutral-200 hover:bg-neutral-800/60'}`}
        aria-current={$page.url.pathname.includes('heal') ? 'page' : undefined}
        onclick={() => goto(resolve('/live/heal'))}
      >HEAL</button>
      <button
        class={`rounded-md ${density === 'comfortable' ? 'px-3 py-1.5' : density === 'medium' ? 'px-2.5 py-1' : 'px-2 py-0.5'} transition-colors font-medium tracking-wide uppercase text-[11px] shadow-sm border ${$page.url.pathname.includes('tanked') ? 'bg-neutral-800/90 text-neutral-100 border-neutral-700' : 'text-neutral-400 border-neutral-800 hover:text-neutral-200 hover:bg-neutral-800/60'}`}
        aria-current={$page.url.pathname.includes('tanked') ? 'page' : undefined}
        onclick={() => goto(resolve('/live/tanked'))}
      >TANKED</button>
    </span>

    <!-- Buttons -->
  <span class="flex items-center {density === 'comfortable' ? 'gap-2.5' : 'gap-2'}">
  <button class="text-neutral-400 hover:text-neutral-200 transition" onclick={handleResetEncounter} {@attach tooltip(() => 'Reset Encounter')}><RefreshCwIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} /></button>
  <button class="{isEncounterPaused ? 'text-neutral-300' : 'text-neutral-400'} hover:text-neutral-200 transition" onclick={() => { togglePauseEncounter(); isEncounterPaused = !isEncounterPaused; }}>
        {#if isEncounterPaused}
          <PlayIcon {@attach tooltip(() => 'Resume Encounter')} class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} />
        {:else}
          <PauseIcon {@attach tooltip(() => 'Pause Encounter')} class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} />
        {/if}
      </button>
      <button class="transition {bossOnlyDpsEnabled ? 'text-yellow-400' : 'text-neutral-400 hover:text-neutral-200'}" class:boss-only-active={bossOnlyDpsEnabled} aria-pressed={bossOnlyDpsEnabled} onclick={toggleBossOnlyDamage} {@attach tooltip(() => (bossOnlyDpsEnabled ? 'Boss Only Damage Enabled' : 'Enable Boss Only Damage'))}>
        <CrownIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4.5' : 'size-4'} />
      </button>
      <button class="transition {density !== 'comfortable' ? 'text-sky-400' : 'text-neutral-400 hover:text-neutral-200'} {densityAnimating ? 'scale-90' : ''}" class:compact-mode-active={density !== 'comfortable'} class:density-animating={densityAnimating} aria-pressed={density !== 'comfortable'} onclick={toggleDensity} {@attach tooltip(() => density === 'comfortable' ? 'Density: Comfortable' : density === 'medium' ? 'Density: Medium' : 'Density: Compact')}>
        <MinimizeIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4.5' : 'size-4'} />
      </button>
  <button class="text-neutral-400 hover:text-neutral-200 transition" onclick={() => appWindow.setIgnoreCursorEvents(true)} {@attach tooltip(() => 'Clickthrough')}><PointerIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} /></button>
  <button class="text-neutral-400 hover:text-neutral-200 transition" onclick={() => openSettings()} {@attach tooltip(() => 'Settings')}><SettingsIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} /></button>
  <button class="text-neutral-400 hover:text-neutral-200 transition" onclick={() => appWindow.hide()} {@attach tooltip(() => 'Minimize')}><MinusIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-5' : 'size-4'} /></button>
    </span>
  </div>

  <!-- Row 2, Col 1: Version + Time Elapsed -->
  <div class="col-start-1 row-start-2 flex items-center {density === 'comfortable' ? 'gap-4' : 'gap-3'}">
    <span class="text-neutral-500 tracking-tight">Resonance Logs v{#await getVersion()}X.Y.Z{:then version}{version}{/await}</span>
    <span class="font-medium text-neutral-300 tabular-nums" {@attach tooltip(() => 'Time Elapsed')}>{formatElapsed(clientElapsedMs)}</span>
  </div>

  <!-- Row 3, Col 1: Stats summary -->
  <div class="col-start-1 row-start-3 flex items-center {density === 'comfortable' ? 'gap-5' : 'gap-3'}">
    <span class="flex items-center gap-1.5 text-neutral-300"><span class="text-neutral-500 uppercase tracking-wide text-[10px]" {@attach tooltip(() => 'Total Damage Dealt')}>T.DMG</span><span class="font-semibold text-neutral-200" {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}><AbbreviatedNumber num={Number(headerInfo.totalDmg)} /></span></span>
    <span class="flex items-center gap-1.5 text-neutral-300"><span class="text-neutral-500 uppercase tracking-wide text-[10px]" {@attach tooltip(() => 'Total Damage per Second')}>T.DPS</span><span class="font-semibold text-neutral-200" {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}><AbbreviatedNumber num={headerInfo.totalDps} /></span></span>
    <span class="flex items-center gap-1.5 text-neutral-300"><span class="text-neutral-500 uppercase tracking-wide text-[10px]" {@attach tooltip(() => 'Number of Players in current tab')}>PLAYERS</span><span class="font-semibold text-neutral-200 tabular-nums">{playersCount}</span></span>
  </div>

  <!-- Row 2-3, Col 2: Boss Health -->
  <div class="col-start-2 row-start-2 row-span-2 justify-self-end pr-2">
    <BossHealth />
  </div>
</header>

<style>
  /* removed bespoke icon toggle styles in favor of tailwind utility classes */
</style>
