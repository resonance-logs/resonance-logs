<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getVersion } from "@tauri-apps/api/app";
</script>

<footer class="sticky bottom-0 h-7 flex items-center justify-between bg-neutral-800/70 px-1.5">
  <span class="flex h-full items-center">
    <!-- TODO: turn this into a loop? -->
    <button
      class={`rounded-xs cursor-pointer px-1.5 ${page.url.pathname.includes("dps") ? "bg-red-500/40" : ""}`}
      onclick={() => {
        goto(resolve("/live/dps"));
      }}>DPS</button
    >
    <button
      class={`rounded-xs cursor-pointer px-1.5 ${page.url.pathname.includes("heal") ? "bg-red-500/40" : ""}`}
      onclick={() => {
        goto(resolve("/live/heal"));
      }}>HEAL (WIP)</button
    >
  </span>
  <span class="px-1.5 tracking-tighter">
    <span>BPSR Logs</span>
    <span>
      {#await getVersion()}
        vX.Y.Z
      {:then version}
        v{version}
      {/await}
    </span>
  </span>
</footer>
