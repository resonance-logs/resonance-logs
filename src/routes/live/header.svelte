<script lang="ts">
  import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow";

  import CameraIcon from "virtual:icons/lucide/camera";
  import TimerResetIcon from "virtual:icons/lucide/timer-reset";
  import PauseIcon from "virtual:icons/lucide/pause";
  import PlayIcon from "virtual:icons/lucide/play";
  import MinusIcon from "virtual:icons/lucide/minus";
  import PointerIcon from "virtual:icons/lucide/pointer";
  import SettingsIcon from "virtual:icons/lucide/settings";
  import RefreshCwIcon from "virtual:icons/lucide/refresh-cw";

  import { onMount, tick } from "svelte";
  import { onEncounterUpdate, resetEncounter, togglePauseEncounter, type HeaderInfo } from "$lib/api";
  import { takeScreenshot, tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import { emitTo } from "@tauri-apps/api/event";
  import { SETTINGS } from "$lib/settings-store";

  onMount(() => {
    let unlisten: (() => void) | null = null;

    onEncounterUpdate((event) => {
      headerInfo = event.payload.headerInfo;
      isEncounterPaused = event.payload.isPaused;
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
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
  });
  let isEncounterPaused = $state(false);
  const {
    screenshotDiv,
  }: {
    screenshotDiv?: HTMLElement;
  } = $props();
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
</script>

<!-- justify-between to create left/right sides -->
<header data-tauri-drag-region class="sticky top-0 flex h-7 w-full items-center justify-between bg-neutral-900/80 px-1">
  <!-- Left side -->
  <span>
    <button
      onclick={() => {
        commands.hardReset();
        window.location.reload();
      }}
      {@attach tooltip(() => "Temp Fix: Hard Reset")}><RefreshCwIcon /></button
    >
    <span {@attach tooltip(() => "Time Elapsed")}>{formatElapsed(headerInfo.elapsedMs)}</span>
    <span><span {@attach tooltip(() => "Total Damage Dealt")}>T.DMG</span> <span {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}><AbbreviatedNumber num={Number(headerInfo.totalDmg)} /></span></span>
    <span><span {@attach tooltip(() => "Total Damage per Second")}>T.DPS</span> <span {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}><AbbreviatedNumber num={headerInfo.totalDps} /></span></span>
  </span>
  <!-- Right side -->
  <span class="flex gap-1">
    <!-- TODO: add responsive clicks, toaster -->
    <button
      onclick={async () => takeScreenshot(screenshotDiv)}
      {@attach tooltip(() => "Screenshot to Clipboard")}
    >
      <CameraIcon />
    </button>
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
