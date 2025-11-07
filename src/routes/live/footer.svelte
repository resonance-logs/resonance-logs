<script lang="ts">
  /**
   * @file This component displays the footer of the live meter,
   * which includes navigation buttons for DPS, Heal, and Tanked views,
   * as well as the application version.
   */
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getVersion } from "@tauri-apps/api/app";
  import { settings } from "$lib/settings-store";

  // Track compact mode
  let density = $derived(settings.state.accessibility.density ?? "comfortable");
</script>

<footer class="flex {density === 'comfortable' ? 'h-10 px-4 text-sm' : density === 'medium' ? 'h-8 px-3 text-[12px]' : 'h-7 px-2 text-[11px]'} items-center justify-between bg-neutral-900 rounded-b-lg">
  <span class="flex h-full items-center gap-1">
    <button
      class={`rounded-md ${density === 'comfortable' ? 'px-3 py-1' : density === 'medium' ? 'px-2.5 py-0.75' : 'px-2 py-0.5'} transition-all font-medium ${page.url.pathname.includes("dps") ? "bg-neutral-700/60 text-white" : "text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50"}`}
      onclick={() => {
        goto(resolve("/live/dps"));
      }}>DPS</button
    >
    <button
      class={`rounded-md ${density === 'comfortable' ? 'px-3 py-1' : density === 'medium' ? 'px-2.5 py-0.75' : 'px-2 py-0.5'} transition-all font-medium ${page.url.pathname.includes("heal") ? "bg-neutral-700/60 text-white" : "text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50"}`}
      onclick={() => {
        goto(resolve("/live/heal"));
      }}>HEAL</button
    >
    <button
      class={`rounded-md ${density === 'comfortable' ? 'px-3 py-1' : density === 'medium' ? 'px-2.5 py-0.75' : 'px-2 py-0.5'} transition-all font-medium ${page.url.pathname.includes("tanked") ? "bg-neutral-700/60 text-white" : "text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800/50"}`}
      onclick={() => {
        goto(resolve("/live/tanked"));
      }}>TANKED</button
    >
  </span>
  <span class="text-neutral-400 tracking-tighter"><span>Resonance Logs v{#await getVersion()}X.Y.Z{:then version}{version}{/await}</span></span>
</footer>
