<script lang="ts">
  import { settings } from "$lib/settings-store";
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";

  interface NetworkAdapter {
    name: string;
    description: string;
  }

  let adapters: NetworkAdapter[] = $state([]);
  let loadingAdapters = $state(false);
  let error = $state<string | null>(null);

  async function fetchAdapters() {
    loadingAdapters = true;
    error = null;
    try {
      adapters = await invoke("get_network_adapters");
      // If current adapter is not in list (and we have a list), maybe reset or warn?
      // For now, just keep it.
    } catch (e) {
      error = String(e);
    } finally {
      loadingAdapters = false;
    }
  }

  function updateConfig() {
    invoke("set_packet_capture_config", {
      config: {
        method: settings.state.packetCapture.method,
        adapter: settings.state.packetCapture.adapter,
      },
    }).catch((e) => {
      console.error("Failed to update packet capture config:", e);
    });
  }

  $effect(() => {
    if (settings.state.packetCapture.method === "Npcap") {
      fetchAdapters();
    }
  });

  // Watch for changes and update backend
  $effect(() => {
    // Trigger update when method or adapter changes
    // We use a timeout to debounce slightly if needed, but direct call is fine for now
    updateConfig();
  });
</script>

<div class="flex flex-col gap-4">
  <div class="flex flex-col gap-2">
    <h3 class="text-lg font-medium">Packet Capture</h3>
    <p class="text-sm text-muted-foreground">
      Configure how the application captures network packets.
    </p>
  </div>

  <div class="flex flex-col gap-2">
    <label for="capture-method" class="text-sm font-medium"
      >Capture Method</label
    >
    <select
      id="capture-method"
      class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
      bind:value={settings.state.packetCapture.method}
    >
      <option value="Windivert">WinDivert (Default)</option>
      <option value="Npcap">Npcap</option>
    </select>
  </div>

  {#if settings.state.packetCapture.method === "Npcap"}
    <div class="flex flex-col gap-2 border-l-2 border-primary/20 pl-4">
      <div class="flex items-center justify-between">
        <span class="text-sm font-medium">Network Adapter</span>
        {#if loadingAdapters}
          <span class="text-xs text-muted-foreground">Loading...</span>
        {/if}
      </div>

      {#if error}
        <p class="text-xs text-red-500">Error listing adapters: {error}</p>
        <div class="flex flex-col gap-1">
          <p class="text-xs text-muted-foreground">
            Make sure Npcap is installed.
          </p>
          <button
            class="w-fit p-0 text-left text-xs font-normal text-primary hover:underline"
            onclick={() => openUrl("https://npcap.com/#download")}
          >
            Download Npcap
          </button>
        </div>
      {:else}
        <select
          class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          bind:value={settings.state.packetCapture.adapter}
        >
          <option value={null}>Auto / Loopback</option>
          {#each adapters as adapter}
            <option value={adapter.name}>{adapter.description}</option>
          {/each}
        </select>
        <p class="text-xs text-muted-foreground">
          Select the network adapter that handles game traffic.
        </p>
      {/if}
    </div>
  {/if}
</div>
