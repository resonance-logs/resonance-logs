<script lang="ts">
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { Minus } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { commands, type HeaderInfo } from "$lib/bindings";
  import { abbreviateNumberSplit } from "$lib/utils";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);

    return () => clearInterval(interval);
  });

  // is there a better way to init these?
  let headerInfo: HeaderInfo = $state({ totalDps: 0, totalDmg: 0n, elapsedMs: 0n });

  async function fetchData() {
    try {
      headerInfo = await commands.getHeaderInfo();
      console.log(+Date.now(), $state.snapshot(headerInfo));
    } catch (e) {
      console.error("Error fetching data:", e);
    }
  }

  const appWindow = getCurrentWebviewWindow();

  function timestampToMinutesAndSeconds(millis: bigint, useText: boolean = false, showMs = false, extraPad = false): string {
    const millis_float = Number(millis);
    const hoursmillis = millis_float % (60 * 60 * 1000);
    const minutes = Math.floor(hoursmillis / (60 * 1000));
    const minutesmillis = millis_float % (60 * 1000);
    const sec = Math.floor(minutesmillis / 1000);

    return useText ? String(minutes).padStart(extraPad ? 2 : 1, "0") + "m" + String(sec).padStart(2, "0") + "s" + (showMs ? String(millis_float % 1000).padStart(3, "0") + "ms" : "") : String(minutes).padStart(extraPad ? 2 : 1, "0") + ":" + String(sec).padStart(2, "0") + (showMs ? "." + String(millis_float % 1000).padStart(3, "0") : "");
  }
</script>

<!-- justify-between to create left/right sides -->
<header data-tauri-drag-region class="sticky top-0 flex h-7 w-full items-center justify-between bg-neutral-900/80 px-1">
  <!-- Left side -->
  <span>
    <span>{timestampToMinutesAndSeconds(headerInfo.elapsedMs)}</span>
    <span>
      <!-- TODO: fix this with real data -->
      <span>T.DMG</span>
      
      <span>{abbreviateNumberSplit(Number(headerInfo.totalDmg))[0]}<span class="text-tiny text-gray-300">{abbreviateNumberSplit(Number(headerInfo.totalDmg))[1]}</span></span>
    </span>
    <span>
      <!-- TODO: fix this with real data -->
      <span>T.DPS</span>
      <span>{abbreviateNumberSplit(headerInfo.totalDps)[0]}<span class="text-tiny text-gray-300">{abbreviateNumberSplit(headerInfo.totalDps)[1]}</span></span>
    </span>
  </span>
  <!-- Right side -->
  <span class="items-center">
    <button class="cursor-pointer" onclick={() => appWindow.hide()}><Minus /></button>
  </span>
</header>
