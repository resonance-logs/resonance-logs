<script lang="ts">
  /**
   * @file This component displays the header of the live meter,
   * which includes encounter statistics, and buttons for controlling the meter.
   */
  import PauseIcon from "virtual:icons/lucide/pause";
  import PlayIcon from "virtual:icons/lucide/play";
  import RefreshCwIcon from "virtual:icons/lucide/refresh-cw";
  import CrownIcon from "virtual:icons/lucide/crown";

  import { onMount } from "svelte";
  import { onEncounterUpdate, onResetEncounter, togglePauseEncounter, setBossOnlyDps, resetEncounter, type HeaderInfo } from "$lib/api";
  import { tooltip } from "$lib/utils.svelte";
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
      if (newFightStartTimestamp > 0 && newFightStartTimestamp !== fightStartTimestampMs) {
        fightStartTimestampMs = newFightStartTimestamp;
        clientElapsedMs = Date.now() - fightStartTimestampMs;
      }

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
  

  function toggleBossOnlyDamage() {
    const nextValue = !SETTINGS.live.general.state.bossOnlyDps;
    SETTINGS.live.general.state.bossOnlyDps = nextValue;
    void setBossOnlyDps(nextValue);
  }

  // When reset encounter button is pressed -> reset state and notify backend
  function handleResetEncounter() {
    resetTimer();
    isEncounterPaused = false;
    void resetEncounter();
  }

</script>

<header
  data-tauri-drag-region
  class="w-full flex justify-between"
>
  <div class="flex gap-2">
    <div class="flex items-center">
      <span class="text-lg font-bold text-foreground tabular-nums tracking-tight leading-none" {@attach tooltip(() => 'Time Elapsed')}>{formatElapsed(clientElapsedMs)}</span>
    </div>
    <div class="flex items-center">
      <span class="text-lg font-medium shrink-0 leading-none" {@attach tooltip(() => headerInfo.sceneName || "")}>{headerInfo.sceneName || "Unknown Scene"}</span>
      {#if headerInfo.bosses.length > 0 || true}
        <span class="text-muted-foreground font-medium shrink-0 leading-none mx-1">-</span>
        <span class="text-base text-muted-foreground font-medium shrink-0 leading-none align-text-bottom">{headerInfo.bosses[0]?.name || "Unknown Boss"}</span>
      {/if}
    </div>
  </div>
  <div>
    <button
    class="text-muted-foreground hover:text-foreground hover:bg-popover/60 rounded-lg p-2 transition-all duration-200"
      onclick={handleResetEncounter}
      aria-label="Reset encounter"
      {@attach tooltip(() => 'Reset Encounter')}
    >
      <RefreshCwIcon class="size-5"/>
    </button>

    <button
      class="{isEncounterPaused ? 'text-[oklch(0.65_0.1_145)] bg-[oklch(0.9_0.02_145)]/30' : 'text-muted-foreground'} hover:text-foreground hover:bg-popover/60 rounded-lg transition-all duration-200"
      onclick={() => { togglePauseEncounter(); isEncounterPaused = !isEncounterPaused; }}
    >
      {#if isEncounterPaused}
        <PlayIcon {@attach tooltip(() => 'Resume Encounter')} class="size-5" />
      {:else}
        <PauseIcon {@attach tooltip(() => 'Pause Encounter')} class="size-5" />
      {/if}
    </button>

    <button
      class="rounded-lg transition-all duration-200 {bossOnlyDpsEnabled ? 'text-[oklch(0.75_0.1_95)] bg-[oklch(0.95_0.02_95)]/30 hover:bg-[oklch(0.95_0.02_95)]/50' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
      class:boss-only-active={bossOnlyDpsEnabled}
      aria-pressed={bossOnlyDpsEnabled}
      onclick={toggleBossOnlyDamage}
      {@attach tooltip(() => (bossOnlyDpsEnabled ? 'Boss Only Damage Enabled' : 'Enable Boss Only Damage'))}
    >
      <CrownIcon class="size-5" />
    </button>
  </div>
</header>

