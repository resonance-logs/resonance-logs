<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getVersion } from "@tauri-apps/api/app";
  import { settings } from "$lib/settings-store";

  // Track compact mode
  let isCompactMode = $derived(settings.state.accessibility.compactMode);
</script>

<footer class="flex {isCompactMode ? 'h-7' : 'h-10'} items-center justify-between bg-neutral-900 {isCompactMode ? 'px-2' : 'px-4'} {isCompactMode ? 'text-[11px]' : 'text-sm'} rounded-b-lg">
  <span class="flex h-full items-center gap-1">
    <button
      class={`rounded-md ${isCompactMode ? 'px-2 py-0.5' : 'px-3 py-1'} transition-all font-medium ${page.url.pathname.includes("dps") ? "bg-neutral-700/60 text-white" : "text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50"}`}
      onclick={() => {
        goto(resolve("/live/dps"));
      }}>DPS</button
    >
    <button
      class={`rounded-md ${isCompactMode ? 'px-2 py-0.5' : 'px-3 py-1'} transition-all font-medium ${page.url.pathname.includes("heal") ? "bg-neutral-700/60 text-white" : "text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50"}`}
      onclick={() => {
        goto(resolve("/live/heal"));
      }}>HEAL</button
    >
    <button
      class={`rounded-md ${isCompactMode ? 'px-2 py-0.5' : 'px-3 py-1'} transition-all font-medium ${page.url.pathname.includes("tanked") ? "bg-neutral-700/60 text-white" : "text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50"}`}
      onclick={() => {
        goto(resolve("/live/tanked"));
      }}>TANKED</button
    >
  </span>
  <span class="text-neutral-400 tracking-tighter"><span>BPSR Logs v{#await getVersion()}X.Y.Z{:then version}{version}{/await}</span></span>
</footer>
