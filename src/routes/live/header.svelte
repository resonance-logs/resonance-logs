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
  import MousePointerClickIcon from "virtual:icons/lucide/mouse-pointer-click";
  import SettingsIcon from "virtual:icons/lucide/settings";
  import RefreshCwIcon from "virtual:icons/lucide/refresh-cw";
  import CrownIcon from "virtual:icons/lucide/crown";
  import MinimizeIcon from "virtual:icons/lucide/minimize-2";
  import EyeOffIcon from "virtual:icons/lucide/eye-off";

  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getVersion } from "@tauri-apps/api/app";
  import { onEncounterUpdate, onResetEncounter, resetEncounter, togglePauseEncounter, setBossOnlyDps, type HeaderInfo } from "$lib/api";
  // import { takeScreenshot, tooltip } from "$lib/utils.svelte";
  import { tooltip, toggleClickthrough } from "$lib/utils.svelte";
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
  // Use live.general for bossOnlyDps; keep density from accessibility store
  let bossOnlyDpsEnabled = $derived(SETTINGS.live.general.state.bossOnlyDps);
  let density = $derived(SETTINGS.accessibility.state.density ?? "comfortable");
  let transparentMode = $derived(SETTINGS.accessibility.state.transparency ?? false);
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

  function toggleTransparentMode() {
    SETTINGS.accessibility.state.transparency = !SETTINGS.accessibility.state.transparency;
  }

  async function handleClickthroughToggle() {
    await toggleClickthrough();
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
  class="grid w-full grid-cols-[1fr_auto] grid-rows-[auto_auto] pb-2
  {density === 'comfortable' ? 'text-sm' : density === 'medium' ? 'text-[13px]' : 'text-xs'}"
>
  <!-- Row 1, Col 1: Version + Timer -->
  <div class="col-start-1 row-start-1 flex items-center overflow-hidden {density === 'comfortable' ? 'gap-4' : 'gap-3'} min-w-0" data-tauri-drag-region>
    <div class="hidden min-[40rem]:flex items-center gap-2 shrink-0">
      <span class="{density === 'comfortable' ? 'text-base' : density === 'medium' ? 'text-sm' : 'text-xs'} font-bold text-foreground tracking-tight leading-none">Resonance Logs</span>
      <span class="{density === 'comfortable' ? 'text-sm' : density === 'medium' ? 'text-xs' : 'text-[11px]'} font-medium text-muted-foreground tracking-tight leading-none">v{#await getVersion()}X.Y.Z{:then version}{version}{/await}</span>
    </div>
  {#if headerInfo.sceneName}
    <div class="hidden min-[48rem]:block h-4 w-px bg-border shrink-0 opacity-60"></div>
    <span class="{density === 'comfortable' ? 'text-base' : density === 'medium' ? 'text-sm' : 'text-xs'} text-muted-foreground font-medium shrink-0 leading-none" {@attach tooltip(() => headerInfo.sceneName || "")}>{headerInfo.sceneName}</span>
  {/if}
  <div class="hidden min-[48rem]:block h-4 w-px bg-border shrink-0 opacity-60"></div>
    <div class="flex items-center gap-2 shrink-0">
  <span class="{density === 'comfortable' ? 'text-sm' : density === 'medium' ? 'text-xs' : 'text-[11px]'} font-medium text-muted-foreground uppercase tracking-wider leading-none">Timer</span>
  <span class="{density === 'comfortable' ? 'text-lg' : density === 'medium' ? 'text-base' : 'text-sm'} font-bold text-foreground tabular-nums tracking-tight leading-none" {@attach tooltip(() => 'Time Elapsed')}>{formatElapsed(clientElapsedMs)}</span>
    </div>
  </div>

  <!-- Row 1, Col 2: Buttons -->
  <div class="col-start-2 row-start-1 flex items-center justify-self-end {density === 'comfortable' ? 'gap-2' : 'gap-0.5'} shrink-0">
    <button
      class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg {density === 'comfortable' ? 'p-2' : 'p-1.5'} transition-all duration-200"
      onclick={handleResetEncounter}
      {@attach tooltip(() => 'Reset Encounter')}
    >
      <RefreshCwIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
    </button>
    <button
      class="{isEncounterPaused ? 'text-[oklch(0.65_0.1_145)] bg-[oklch(0.9_0.02_145)]/30' : 'text-muted-foreground'} hover:text-foreground hover:bg-popover/60 rounded-lg {density === 'comfortable' ? 'p-2' : 'p-1.5'} transition-all duration-200"
      onclick={() => { togglePauseEncounter(); isEncounterPaused = !isEncounterPaused; }}
    >
      {#if isEncounterPaused}
        <PlayIcon {@attach tooltip(() => 'Resume Encounter')} class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
      {:else}
        <PauseIcon {@attach tooltip(() => 'Pause Encounter')} class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
      {/if}
    </button>
    <button
      class="rounded-lg {density === 'comfortable' ? 'p-2' : 'p-1.5'} transition-all duration-200 {bossOnlyDpsEnabled ? 'text-[oklch(0.75_0.1_95)] bg-[oklch(0.95_0.02_95)]/30 hover:bg-[oklch(0.95_0.02_95)]/50' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
      class:boss-only-active={bossOnlyDpsEnabled}
      aria-pressed={bossOnlyDpsEnabled}
      onclick={toggleBossOnlyDamage}
      {@attach tooltip(() => (bossOnlyDpsEnabled ? 'Boss Only Damage Enabled' : 'Enable Boss Only Damage'))}
    >
      <CrownIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
    </button>
    <button
      class="rounded-lg {density === 'comfortable' ? 'p-2' : 'p-1.5'} transition-all duration-200 {transparentMode ? 'text-[oklch(0.65_0.12_280)] bg-[oklch(0.9_0.03_280)]/30 hover:bg-[oklch(0.9_0.03_280)]/50' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
      aria-pressed={transparentMode}
      onclick={toggleTransparentMode}
      {@attach tooltip(() => (transparentMode ? 'Transparent Mode Enabled' : 'Enable Transparent Mode'))}
    >
      <EyeOffIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
    </button>
    <button
      class="rounded-lg {density === 'comfortable' ? 'p-2' : 'p-1.5'} transition-all duration-200 {densityAnimating ? 'scale-90' : ''} {density !== 'comfortable' ? 'text-[oklch(0.6_0.1_220)] bg-[oklch(0.9_0.02_220)]/30 hover:bg-[oklch(0.9_0.02_220)]/50' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
      class:compact-mode-active={density !== 'comfortable'}
      class:density-animating={densityAnimating}
      aria-pressed={density !== 'comfortable'}
      onclick={toggleDensity}
      {@attach tooltip(() => density === 'comfortable' ? 'Density: Comfortable' : density === 'medium' ? 'Density: Medium' : 'Density: Compact')}
    >
      <MinimizeIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
    </button>
    <button
      class="rounded-lg {density === 'comfortable' ? 'p-2' : 'p-1.5'} transition-all duration-200 text-muted-foreground hover:text-foreground hover:bg-popover/60"
      onclick={handleClickthroughToggle}
      {@attach tooltip(() => 'Toggle Clickthrough Mode')}
    >
      <MousePointerClickIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
    </button>
    <div class="h-5 w-px bg-neutral-700/60"></div>
    <!-- <button
      class="text-neutral-400 hover:text-neutral-100 hover:bg-neutral-800/60 rounded-lg p-1.5 transition-all duration-200"
      onclick={() => appWindow.setIgnoreCursorEvents(true)}
      {@attach tooltip(() => 'Clickthrough')}
    >
      <PointerIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
    </button> -->
    <button
      class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg {density === 'comfortable' ? 'p-2' : 'p-1.5'} transition-all duration-200"
      onclick={() => openSettings()}
      {@attach tooltip(() => 'Settings')}
    >
      <SettingsIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
    </button>
    <button
      class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg {density === 'comfortable' ? 'p-2' : 'p-1.5'} transition-all duration-200"
      onclick={() => appWindow.hide()}
      {@attach tooltip(() => 'Minimize')}
    >
      <MinusIcon class={density === 'comfortable' ? 'size-5' : density === 'medium' ? 'size-4' : 'size-3.5'} />
    </button>
  </div>

  <!-- Row 2, Col 1: Stats summary + Boss Health -->
  <div class="col-start-1 row-start-2 flex overflow-hidden items-center {density === 'comfortable' ? 'gap-5' : 'gap-4'} min-w-0">
    <!-- Stats -->
    <div class="hidden min-[32rem]:flex overflow-hidden items-center {density === 'comfortable' ? 'gap-5' : 'gap-4'}">
      <div class="flex items-center gap-2 shrink-0">
        <span class="{density === 'comfortable' ? 'text-base' : density === 'medium' ? 'text-xs' : 'text-[11px]'} font-bold text-muted-foreground uppercase tracking-wider" {@attach tooltip(() => 'Total Damage Dealt')}>T.DMG</span>
        <span class="{density === 'comfortable' ? 'text-lg' : density === 'medium' ? 'text-base' : 'text-sm'} font-bold text-foreground" {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}><AbbreviatedNumber num={Number(headerInfo.totalDmg)} /></span>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <span class="{density === 'comfortable' ? 'text-base' : density === 'medium' ? 'text-xs' : 'text-[11px]'} font-bold text-muted-foreground uppercase tracking-wider" {@attach tooltip(() => 'Total Damage per Second')}>T.DPS</span>
        <span class="{density === 'comfortable' ? 'text-lg' : density === 'medium' ? 'text-base' : 'text-sm'} font-bold text-foreground" {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}><AbbreviatedNumber num={headerInfo.totalDps} /></span>
      </div>
    </div>

    <!-- Divider -->
  <div class="hidden min-[48rem]:block h-5 w-px bg-border shrink-0 opacity-60"></div>

    <!-- Boss Health -->
    <div class="flex items-center gap-2 shrink-0">
  <span class="hidden min-[48rem]:block {density === 'comfortable' ? 'text-base' : density === 'medium' ? 'text-xs' : 'text-[11px]'} font-bold text-muted-foreground uppercase tracking-wider" {@attach tooltip(() => 'Total Damage per Second')}>BOSS</span>
      <BossHealth />
    </div>
  </div>

  <!-- Row 2, Col 2: DPS/HEAL/TANKED Tabs (Connected) -->
  <div class="col-start-2 row-start-2 justify-self-end flex items-center border border-border rounded-lg overflow-hidden bg-popover/30 shrink-0 m-1">
    <button
      class={`${density === 'comfortable' ? 'px-3.5 py-1.5' : density === 'medium' ? 'px-3 py-1' : 'px-2 py-0.75'} transition-all duration-200 font-bold tracking-wider uppercase text-[11px] border-r border-border whitespace-nowrap ${$page.url.pathname.includes('dps') ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}`}
      aria-current={$page.url.pathname.includes('dps') ? 'page' : undefined}
      onclick={() => goto(resolve('/live/dps'))}
    >DPS</button>
    <button
      class={`${density === 'comfortable' ? 'px-3.5 py-1.5' : density === 'medium' ? 'px-3 py-1' : 'px-2 py-0.75'} transition-all duration-200 font-bold tracking-wider uppercase text-[11px] border-r border-border whitespace-nowrap ${$page.url.pathname.includes('heal') ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}`}
      aria-current={$page.url.pathname.includes('heal') ? 'page' : undefined}
      onclick={() => goto(resolve('/live/heal'))}
    >HEAL</button>
    <button
      class={`${density === 'comfortable' ? 'px-3.5 py-1.5' : density === 'medium' ? 'px-3 py-1' : 'px-2 py-0.75'} transition-all duration-200 font-bold tracking-wider uppercase text-[11px] whitespace-nowrap ${$page.url.pathname.includes('tanked') ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}`}
      aria-current={$page.url.pathname.includes('tanked') ? 'page' : undefined}
      onclick={() => goto(resolve('/live/tanked'))}
    >TANKED</button>
  </div>
</header>

<style>
  /* removed bespoke icon toggle styles in favor of tailwind utility classes */
</style>
