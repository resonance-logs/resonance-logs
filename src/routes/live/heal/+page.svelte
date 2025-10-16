<script lang="ts">
  import { onMount } from "svelte";
  import { onHealPlayersUpdate, type PlayersWindow } from "$lib/api";
  import { getClassColor } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { getCoreRowModel } from "@tanstack/table-core";
  import { createSvelteTable } from "$lib/svelte-table";
  import { healPlayersColumnDefs } from "$lib/table-info";
  import FlexRender from "$lib/svelte-table/flex-render.svelte";
  import { settings } from "$lib/settings-store";

  let unlisten: (() => void) | null = null;
  let lastEventTime = Date.now();
  let reconnectInterval: ReturnType<typeof setInterval> | null = null;
  let isReconnecting = false;
  const RECONNECT_DELAY = 1000;
  const DISCONNECT_THRESHOLD = 5000;

  async function setupEventListener() {
    if (isReconnecting) return;
    try {
      const unlistenFn = await onHealPlayersUpdate((event) => {
        lastEventTime = Date.now();
        healPlayersWindow = event.payload;
      });
      unlisten = unlistenFn;
      console.log("Event listener set up for heal players");
    } catch (e) {
      console.error("Failed to set up event listener for heal players:", e);
      isReconnecting = true;
      setTimeout(() => {
        isReconnecting = false;
        setupEventListener();
      }, RECONNECT_DELAY);
    }
  }

  function startReconnectCheck() {
    reconnectInterval = setInterval(() => {
      if (Date.now() - lastEventTime > DISCONNECT_THRESHOLD) {
        console.warn("Event stream disconnected for heal players, attempting reconnection");
        if (unlisten) {
          unlisten();
          unlisten = null;
        }
        setupEventListener();
      }
    }, 1000);
  }

  onMount(() => {
    setupEventListener();
    startReconnectCheck();

    return () => {
      if (reconnectInterval) clearInterval(reconnectInterval);
      if (unlisten) unlisten();
    };
  });

  let healPlayersWindow: PlayersWindow = $state({ playerRows: [] });

  const healTable = createSvelteTable({
    get data() {
      return healPlayersWindow.playerRows;
    },
    columns: healPlayersColumnDefs,
    getCoreRowModel: getCoreRowModel(),
    state: {
      get columnVisibility() {
        return settings.state["live"]["heal"]["players"];
      },
    },
  });

  let SETTINGS_YOUR_NAME = $derived(settings.state["general"]["showYourName"]);
  let SETTINGS_OTHERS_NAME = $derived(settings.state["general"]["showOthersName"]);
</script>

<div class="relative flex flex-col">
  <table class="w-screen table-fixed">
    <thead class="z-1 sticky top-0 h-6">
      <tr class="bg-neutral-900">
        {#each healTable.getHeaderGroups() as headerGroup (headerGroup.id)}
          {#each headerGroup.headers as header (header.id)}
            <th class={header.column.columnDef.meta?.class}><FlexRender content={header.column.columnDef.header ?? "UNKNOWN HEADER"} context={header.getContext()} /></th>
          {/each}
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each healTable.getRowModel().rows as row (row.id)}
        {@const className = row.original.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? row.original.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? row.original.className : ""}
        <tr class="h-7 px-2 py-1 text-center" onclick={() => goto(`/live/heal/skills?playerUid=${row.original.uid}`)}>
          {#each row.getVisibleCells() as cell (cell.id)}
            <td><FlexRender content={cell.column.columnDef.cell ?? "UNKNOWN CELL"} context={cell.getContext()} /></td>
          {/each}
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(className)}; width: {row.original.dmgPct}vw;"></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
