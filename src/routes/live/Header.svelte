<script lang="ts">
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { RefreshCw, Camera, Pause, Play, Minus } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { commands, type HeaderInfo } from "$lib/bindings";
  import { takeScreenshot, tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "$lib/components/AbbreviatedNumber.svelte";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);

    return () => clearInterval(interval);
  });

  async function fetchData() {
    try {
      const result = await commands.getHeaderInfo();
      if (result.status !== "ok") {
        console.warn("Failed to get header: ", result.error);
        return;
      } else {
        headerInfo = result.data;
        console.log("header: ", +Date.now(), $state.snapshot(headerInfo));
      }
    } catch (e) {
      console.error("Error fetching data:", e);
    }
  }

  function formatElapsed(msBigInt: bigint) {
    const totalSeconds = Math.floor(Number(msBigInt) / 1000);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }

  let headerInfo: HeaderInfo = $state({ totalDps: 0, totalDmg: 0n, elapsedMs: 0n });
  let isEncounterPaused = $state(false);
  const { screenshotDiv }: { screenshotDiv?: HTMLElement } = $props();
  const appWindow = getCurrentWebviewWindow();
</script>

<!-- justify-between to create left/right sides -->
<header data-tauri-drag-region class="sticky top-0 flex h-7 w-full items-center justify-between bg-neutral-900/80 px-1">
  <!-- Left side -->
  <span>
    <span {@attach tooltip(() => "Time Elapsed")}>{formatElapsed(headerInfo.elapsedMs)}</span>
    <span><span {@attach tooltip(() => "Total Damage Dealt")}>T.DMG</span> <span {@attach tooltip(() => headerInfo.totalDmg.toLocaleString())}><AbbreviatedNumber val={Number(headerInfo.totalDmg)} /></span></span>
    <span><span {@attach tooltip(() => "Total Damage per Second")}>T.DPS</span> <span {@attach tooltip(() => headerInfo.totalDps.toLocaleString())}><AbbreviatedNumber val={headerInfo.totalDps} /></span></span>
  </span>
  <!-- Right side -->
  <span class="flex gap-1">
    <!-- TODO: add responsive clicks, toaster -->
    <button onclick={() => commands.resetEncounter()} {@attach tooltip(() => "Reset Encounter")}><RefreshCw /></button>
    <button
      onclick={() => {
        commands.togglePauseEncounter();
        isEncounterPaused = !isEncounterPaused;
      }}
      
    >
      {#if isEncounterPaused}
        <Play {@attach tooltip(() => "Resume Encounter")} />
      {:else}
        <Pause {@attach tooltip(() => "Pause Encounter")} />
      {/if}
    </button>
    <button onclick={() => takeScreenshot(screenshotDiv)} {@attach tooltip(() => "Screenshot to Clipboard")}><Camera /></button>
    <button onclick={() => appWindow.hide()} {@attach tooltip(() => "Minimize")}><Minus /></button>
  </span>
</header>
