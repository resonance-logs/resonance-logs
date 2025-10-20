<script lang="ts">
  import { commands } from "$lib/bindings";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import { getClassColor } from "$lib/utils.svelte";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { createSvelteTable } from "$lib/svelte-table";
  import FlexRender from "$lib/svelte-table/flex-render.svelte";
  import { settings } from "$lib/settings-store";
  import type { SkillsWindow, SkillsUpdatePayload } from "$lib/api";
  import { tankedSkillsColumnDefs } from "$lib/table-info";
  import { onTankedSkillsUpdate } from "$lib/api";
  import { getTankedPlayers } from "$lib/stores/live-meter-store.svelte";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";

  let skillsWindow = $state<SkillsWindow | null>(null);
  let unlisten: (() => void) | null = null;

  // Get the playerUid from the URL query parameters
  const playerUid = Number($page.url.searchParams.get("playerUid"));

  // Create reactive data references to avoid recreating table on every render
  let skillRowsData: any[] = $state([]);
  let columnVisibility = $state(settings.state.live.tanked?.skills || {});

  // Update data when skillsWindow changes
  $effect(() => {
    skillRowsData = skillsWindow?.skillRows ?? [];
  });

  // Update column visibility when settings change
  $effect(() => {
    columnVisibility = settings.state.live.tanked?.skills || {};
  });

  // Define the table reactively when data changes
  const tankedSkillsTable = $derived(createSvelteTable({
    data: skillRowsData,
    columns: tankedSkillsColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      columnVisibility,
    },
  }));

  onMount(() => {
    if (playerUid) {
      // Fetch initial skills data
      commands.getPlayerSkills(playerUid, "tanked").then(result => {
        if (result.status === "ok") {
          skillsWindow = result.data;
        }
      }).catch(e => {
        console.error("Failed to load tanked skills:", e);
      });

      // Listen for updates
      onTankedSkillsUpdate((event: { payload: SkillsUpdatePayload }) => {
        if (event.payload.playerUid === playerUid) {
          skillsWindow = event.payload.skillsWindow;
        }
      }).then(fn => {
        unlisten = fn;
      });
    }

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  });

  // Optimize derived calculations to avoid recalculation on every render
  let maxTakenSkill = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state["general"]["showYourName"]);
  let SETTINGS_OTHERS_NAME = $state(settings.state["general"]["showOthersName"]);

  // Update maxTakenSkill when data changes
  $effect(() => {
    maxTakenSkill = skillsWindow?.skillRows.reduce((max, s) => (s.totalDmg > max ? s.totalDmg : max), 0) ?? 0;
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state["general"]["showYourName"];
    SETTINGS_OTHERS_NAME = settings.state["general"]["showOthersName"];
  });

  // Get players list for breadcrumb info
  let tankedPlayers: any[] = $state([]);
  let currentPlayer: any = $state(null);

  // Update players list when store changes
  $effect(() => {
    tankedPlayers = getTankedPlayers().playerRows;
    currentPlayer = tankedPlayers.find((p) => p.uid === playerUid);
  });
</script>

<!-- Breadcrumb to go back to the main table -->
{#if currentPlayer}
  {@const className = currentPlayer.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currentPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? currentPlayer.className : ""}
  <div
    class="sticky top-0 z-10 flex h-8 w-full items-center gap-2 bg-neutral-900 px-2"
    style="background-color: {getClassColor(className)};"
  >
    <button class="underline" onclick={() => goto("/live/tanked")}>Back</button>
    <span class="font-bold">{currentPlayer.name}</span>
    <span>{currentPlayer.classSpecName}</span>
    <span class="ml-auto">
      <span class="text-xs">Total: </span>
      <AbbreviatedNumber num={currentPlayer.totalDmg} />
    </span>
  </div>
{/if}

<div class="relative flex flex-col">
  <table class="w-screen table-fixed">
    <thead class="z-1 sticky top-0 h-6">
      <tr class="bg-neutral-900">
        {#each tankedSkillsTable.getHeaderGroups() as headerGroup (headerGroup.id)}
          {#each headerGroup.headers as header (header.id)}
            <th class={header.column.columnDef.meta?.class}>
              <FlexRender
                content={header.column.columnDef.header ?? "UNKNOWN HEADER"}
                context={header.getContext()}
              />
            </th>
          {/each}
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each tankedSkillsTable.getRowModel().rows as row (row.id)}
        {@const className = currentPlayer?.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currentPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" && currentPlayer ? currentPlayer.className : ""}
        <tr class="h-7 px-2 py-1 text-center">
          {#each row.getVisibleCells() as cell (cell.id)}
            <td class="text-right">
              <FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} />
            </td>
          {/each}
          <td
            class="-z-1 absolute left-0 h-7"
            style="background-color: {getClassColor(className)}; width: {settings.state.general
              .relativeToTopDPSPlayer
              ? maxTakenSkill > 0
                ? (row.original.totalDmg / maxTakenSkill) * 100
                : 0
              : row.original.dmgPct}%;"
          ></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

