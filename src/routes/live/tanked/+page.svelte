<script lang="ts">
  import { getClassColor } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { createSvelteTable } from "$lib/svelte-table";
  import { tankedPlayersColumnDefs } from "$lib/table-info";
  import FlexRender from "$lib/svelte-table/flex-render.svelte";
  import { settings } from "$lib/settings-store";
  import { getTankedPlayers } from "$lib/stores/live-meter-store.svelte";

  // Create reactive data reference to avoid recreating table on every render
  let tankedData = $state(getTankedPlayers().playerRows);
  let columnVisibility = $state(settings.state.live.tanked?.players || {});

  // Update data when store changes
  $effect(() => {
    tankedData = getTankedPlayers().playerRows;
  });

  // Update column visibility when settings change
  $effect(() => {
    columnVisibility = settings.state.live.tanked?.players || {};
  });

  // Create table reactively when data or visibility changes
  const tankedTable = $derived(createSvelteTable({
    data: tankedData,
    columns: tankedPlayersColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      columnVisibility,
    },
  }));

  // Optimize derived calculations to avoid recalculation on every render
  let maxTaken = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state["general"]["showYourName"]);
  let SETTINGS_OTHERS_NAME = $state(settings.state["general"]["showOthersName"]);

  // Update maxTaken when data changes
  $effect(() => {
    const players = getTankedPlayers().playerRows;
    maxTaken = players.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0);
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state["general"]["showYourName"];
    SETTINGS_OTHERS_NAME = settings.state["general"]["showOthersName"];
  });

</script>

<div class="relative flex flex-col">
  <table class="w-screen table-fixed">
    <thead class="z-1 sticky top-0 h-6">
      <tr class="bg-neutral-900">
        {#each tankedTable.getHeaderGroups() as headerGroup (headerGroup.id)}
          {#each headerGroup.headers as header (header.id)}
            <th class={header.column.columnDef.meta?.class}><FlexRender content={header.column.columnDef.header ?? "UNKNOWN HEADER"} context={header.getContext()} /></th>
          {/each}
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each tankedTable.getRowModel().rows as row (row.id)}
        {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? row.original.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? row.original.className : ""}
        <tr class="h-7 px-2 py-1 text-center" onclick={() => goto(`/live/tanked/skills?playerUid=${row.original.uid}`)}>
          {#each row.getVisibleCells() as cell (cell.id)}
            <td class="text-right"><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
          {/each}
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(className)}; width: {settings.state.general.relativeToTopDPSPlayer ? (maxTaken > 0 ? (row.original.totalDmg / maxTaken) * 100 : 0) : row.original.dmgPct}%;"></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

