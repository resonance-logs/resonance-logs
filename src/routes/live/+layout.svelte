<script lang="ts">
  import { onMount } from "svelte";
  import { commands } from "$lib/bindings";
  import { SETTINGS } from "$lib/settings-store";
  import { cn } from "$lib/utils";
  import { onDpsPlayersUpdate, onHealPlayersUpdate, onResetEncounter } from "$lib/api";
  import { setDpsPlayers, setHealPlayers, clearMeterData } from "$lib/stores/live-meter-store.svelte";
  import Footer from "./footer.svelte";
  import Header from "./header.svelte";

  let { children } = $props();
  let screenshotDiv: HTMLDivElement | undefined = $state();

  let unlisten: (() => void) | null = null;
  let lastEventTime = Date.now();
  let reconnectInterval: ReturnType<typeof setInterval> | null = null;
  let isReconnecting = false;
  const RECONNECT_DELAY = 1000;
  const DISCONNECT_THRESHOLD = 5000;

  async function setupEventListeners() {
    if (isReconnecting) return;
    try {
      // Set up DPS players listener
      const dpsUnlisten = await onDpsPlayersUpdate((event) => {
        lastEventTime = Date.now();
        setDpsPlayers(event.payload);
      });

      // Set up heal players listener
      const healUnlisten = await onHealPlayersUpdate((event) => {
        lastEventTime = Date.now();
        setHealPlayers(event.payload);
      });

      // Set up reset encounter listener
      const resetUnlisten = await onResetEncounter(() => {
        clearMeterData();
      });

      // Combine all unlisten functions
      unlisten = () => {
        dpsUnlisten();
        healUnlisten();
        resetUnlisten();
      };

      console.log("Event listeners set up for live meter data");
    } catch (e) {
      console.error("Failed to set up event listeners:", e);
      isReconnecting = true;
      setTimeout(() => {
        isReconnecting = false;
        setupEventListeners();
      }, RECONNECT_DELAY);
    }
  }

  function startReconnectCheck() {
    reconnectInterval = setInterval(() => {
      if (Date.now() - lastEventTime > DISCONNECT_THRESHOLD) {
        console.warn("Event stream disconnected, attempting reconnection");
        if (unlisten) {
          unlisten();
          unlisten = null;
        }
        setupEventListeners();
      }
    }, 1000);
  }

  onMount(() => {
    setupEventListeners();
    startReconnectCheck();

    return () => {
      if (reconnectInterval) clearInterval(reconnectInterval);
      if (unlisten) unlisten();
    };
  });

  $effect(() => {
    if (SETTINGS.accessibility.state.blur) {
      commands.enableBlur();
    } else {
      commands.disableBlur();
    }
  });
</script>

<!-- flex flex-col min-h-screen → makes the page stretch full height and stack header, body, and footer. -->
<!-- flex-1 on <main> → makes the body expand to fill leftover space, pushing the footer down. -->
<div class="flex h-screen flex-col text-sm text-white" bind:this={screenshotDiv}>
  <Header {screenshotDiv} />
  <main class={cn("flex-1 overflow-y-auto", !SETTINGS.accessibility.state.transparency && "bg-neutral-900/25")}>
    {@render children()}
  </main>
  <Footer />
</div>

<style>
  :global {
    /* Hide scrollbars globally but keep scrolling functional */
    * {
      -ms-overflow-style: none; /* IE and Edge */
      scrollbar-width: none; /* Firefox */
    }
    *::-webkit-scrollbar {
      display: none; /* Chrome, Safari, Edge */
    }
  }
</style>
