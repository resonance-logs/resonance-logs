<script lang="ts">
  import { onMount } from "svelte";
  import { getClassColor } from "$lib/utils.svelte";
  import { page } from "$app/state";
  import { createSvelteTable, FlexRender } from "$lib/svelte-table";
  import { dpsPlayersColumnDefs, dpsSkillsColumnDefs } from "$lib/table-info";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { settings } from "$lib/settings-store";
  import { commands } from "$lib/bindings";
  import { onDpsSkillsUpdate } from "$lib/api";
  import type { SkillsWindow, SkillsUpdatePayload } from "$lib/api";

  const playerUid: string = page.url.searchParams.get("playerUid") ?? "-1";

  let dpsSkillBreakdownWindow: SkillsWindow = $state({ currPlayer: [], skillRows: [] });
  let unlisten: (() => void) | null = null;

  // Create reactive data references to avoid recreating tables on every render
  let currPlayerData = $state(dpsSkillBreakdownWindow.currPlayer);
  let skillRowsData = $state(dpsSkillBreakdownWindow.skillRows);
  let columnVisibility = $state(settings.state.live.dps.skillBreakdown);

  // Update data when window changes
  $effect(() => {
    currPlayerData = dpsSkillBreakdownWindow.currPlayer;
    skillRowsData = dpsSkillBreakdownWindow.skillRows;
  });

  // Update column visibility when settings change
  $effect(() => {
    columnVisibility = settings.state.live.dps.skillBreakdown;
  });

  // Create tables reactively when data or visibility changes
  const currPlayerTable = $derived(createSvelteTable({
    data: currPlayerData,
    columns: dpsPlayersColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      columnVisibility,
    },
  }));

  const dpsSkillBreakdownTable = $derived(createSvelteTable({
    data: skillRowsData,
    columns: dpsSkillsColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      columnVisibility,
    },
  }));

  // Optimize derived calculations to avoid recalculation on every render
  let maxSkillValue = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state["general"]["showYourName"]);
  let SETTINGS_OTHERS_NAME = $state(settings.state["general"]["showOthersName"]);

  // Update maxSkillValue when data changes
  $effect(() => {
    maxSkillValue = dpsSkillBreakdownWindow.skillRows.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0);
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state["general"]["showYourName"];
    SETTINGS_OTHERS_NAME = settings.state["general"]["showOthersName"];
  });

  async function subscribePlayerSkills() {
    try {
      // Subscribe and get initial data
      const result = await commands.subscribePlayerSkills(parseInt(playerUid), "dps");
      if (result.status === "ok") {
        dpsSkillBreakdownWindow = result.data;
      } else {
        console.error("Failed to subscribe to player skills:", result.error);
      }

      // Set up websocket listener for updates
      unlisten = await onDpsSkillsUpdate((event: { payload: SkillsUpdatePayload }) => {
        // Only update if this is the correct player
        if (event.payload.playerUid.toString() === playerUid) {
          dpsSkillBreakdownWindow = event.payload.skillsWindow;
        }
      });
    } catch (error) {
      console.error("Failed to subscribe to player skills:", error);
    }
  }

  async function unsubscribePlayerSkills() {
    try {
      // Unsubscribe from backend
      await commands.unsubscribePlayerSkills(parseInt(playerUid), "dps");

      // Remove websocket listener
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
    } catch (error) {
      console.error("Failed to unsubscribe from player skills:", error);
    }
  }

  onMount(() => {
    subscribePlayerSkills();

    return () => {
      unsubscribePlayerSkills();
    };
  });
</script>

<svelte:window oncontextmenu={() => window.history.back()} />

<!-- TODO: looks ugly when split, need to figure out logic to combine together https://imgur.com/COalJFe -->
<div class="relative flex flex-col">
  <table class="w-screen table-fixed">
    <thead class="z-1 sticky top-0 h-6">
      <tr class="bg-neutral-900">
        {#each dpsSkillBreakdownTable.getHeaderGroups() as headerGroup (headerGroup.id)}
          {#each headerGroup.headers as header (header.id)}
            <th class={header.column.columnDef.meta?.class}><FlexRender content={header.column.columnDef.header ?? "UNKNOWN HEADER"} context={header.getContext()} /></th>
          {/each}
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each currPlayerTable.getRowModel().rows as row (row.id)}
        {@const currPlayer = dpsSkillBreakdownWindow.currPlayer[0]}
        {#if currPlayer}
          {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? currPlayer.className : ""}
          <tr class="h-7 px-2 py-1 text-center">
            {#each row.getVisibleCells() as cell (cell.id)}
              <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
            {/each}
            <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(className)}; width: 100vw;"></td>
          </tr>
        {/if}
      {/each}
      {#each dpsSkillBreakdownTable.getRowModel().rows as row, i (row.id)}
        {@const currPlayer = dpsSkillBreakdownWindow.currPlayer[0]}
        {#if currPlayer}
          {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? currPlayer.className : ""}
          <tr class="h-7 px-2 py-1 text-center">
            {#each row.getVisibleCells() as cell (cell.id)}
              <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
            {/each}
            <td class="-z-1 absolute left-0 h-7" style="background-color: {`color-mix(in srgb, ${getClassColor(className)} 80%, white ${i % 2 === 0 ? '50%' : '20%'})`}; width: {maxSkillValue > 0 ? (Number(row.original.totalDmg) / Number(maxSkillValue)) * 100 : 0}%;"></td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>
