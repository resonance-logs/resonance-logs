<script lang="ts">
  import { onMount } from "svelte";
  import { commands } from "$lib/bindings";
  import { SETTINGS } from "$lib/settings-store";
  import { cn } from "$lib/utils";
  import { onPlayersUpdate, onResetEncounter, onEncounterUpdate } from "$lib/api";
import { writable } from "svelte/store";
// Store for pause state
export const isPaused = writable(false);
  import { setDpsPlayers, setHealPlayers, setTankedPlayers, clearMeterData } from "$lib/stores/live-meter-store.svelte";
  import Footer from "./footer.svelte";
  import Header from "./header.svelte";
  import NotificationToast from "./notification-toast.svelte";

  let { children } = $props();
  // let screenshotDiv: HTMLDivElement | undefined = $state();

  let notificationToast: NotificationToast;
  let unlisten: (() => void) | null = null;
  let lastEventTime = Date.now();
  let reconnectInterval: ReturnType<typeof setInterval> | null = null;
  let isReconnecting = false;
  const RECONNECT_DELAY = 1000;
  const DISCONNECT_THRESHOLD = 5000;

  async function setupEventListeners() {
    if (isReconnecting) return;
    try {
      // Set up unified players listener
      const playersUnlisten = await onPlayersUpdate((event) => {
        console.log("players websocket", event.payload)
        lastEventTime = Date.now();

        if (event.payload.metricType === "dps") {
          setDpsPlayers(event.payload.playersWindow);
        } else if (event.payload.metricType === "heal") {
          setHealPlayers(event.payload.playersWindow);
        } else if (event.payload.metricType === "tanked") {
          setTankedPlayers(event.payload.playersWindow);
        }
      });

      // Set up reset encounter listener
      const resetUnlisten = await onResetEncounter(() => {
        clearMeterData();
        notificationToast?.showToast('notice', 'Server change detected, resetting log');
      });

      // Set up encounter update listener (pause/resume)
          // track previous pause state locally to avoid spamming toasts on every update
          let lastPauseState: boolean | null = null;
          const encounterUnlisten = await onEncounterUpdate((event) => {
            const newPaused = event.payload.isPaused;
            // update the store regardless
            isPaused.set(newPaused);
            // only show a toast if the pause state actually changed
            if (lastPauseState === null || lastPauseState !== newPaused) {
              if (newPaused) {
                notificationToast?.showToast('notice', 'Encounter paused');
              } else {
                notificationToast?.showToast('notice', 'Encounter resumed');
              }
            }
            lastPauseState = newPaused;
          });

      // Combine all unlisten functions
      unlisten = () => {
        playersUnlisten();
        resetUnlisten();
        encounterUnlisten();
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
<div class="flex h-screen flex-col text-sm text-white">
  <Header />
  <main class={cn("flex-1 overflow-y-auto", !SETTINGS.accessibility.state.transparency && "bg-neutral-900/25")}>
    {@render children()}
  </main>
  <Footer />
  <NotificationToast bind:this={notificationToast} />
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
