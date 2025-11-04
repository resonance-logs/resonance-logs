<script lang="ts">
  import { onMount } from "svelte";
  import { commands } from "$lib/bindings";
  import { SETTINGS } from "$lib/settings-store";
  import { cn } from "$lib/utils";
  import { onPlayersUpdate, onResetEncounter, onEncounterUpdate, onBossDeath, onSceneChange } from "$lib/api";
  import { writable } from "svelte/store";
  import { beforeNavigate, afterNavigate } from "$app/navigation";
  import { page } from "$app/stores";

  // Store for pause state
  export const isPaused = writable(false);

  // Store for scroll positions
  const scrollPositions = writable<Record<string, number>>({});

  import { setDpsPlayers, setHealPlayers, setTankedPlayers, clearMeterData, cleanupStores } from "$lib/stores/live-meter-store.svelte";
  import Footer from "./footer.svelte";
  import Header from "./header.svelte";
  import BossHealth from "./boss-health.svelte";
  import NotificationToast from "./notification-toast.svelte";

  let { children } = $props();
  // let screenshotDiv: HTMLDivElement | undefined = $state();

  let notificationToast: NotificationToast;
  let mainElement: HTMLElement | null = null;
  let unlisten: (() => void) | null = null;
  let lastEventTime = Date.now();
  let reconnectInterval: ReturnType<typeof setInterval> | null = null;
  let isReconnecting = false;
  const RECONNECT_DELAY = 1000;
  const DISCONNECT_THRESHOLD = 5000;

  async function setupEventListeners() {
    if (isReconnecting) return;

    // Clean up existing listeners before setting up new ones
    if (unlisten) {
      unlisten();
      unlisten = null;
    }

    try {
      // Set up unified players listener
      const playersUnlisten = await onPlayersUpdate((event) => {
        // console.log("players websocket", event.payload)
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
        // Treat encounter updates as keep-alive too so reconnect logic doesn't fire
        lastEventTime = Date.now();
        const newPaused = event.payload.isPaused;
        const elapsedMs = event.payload.headerInfo.elapsedMs;
        // update the store regardless
        isPaused.set(newPaused);
        // only show a toast if the pause state actually changed AND we've started receiving combat data
        if (elapsedMs > 0 && (lastPauseState === null || lastPauseState !== newPaused)) {
          if (newPaused) {
            notificationToast?.showToast('notice', 'Encounter paused');
          } else {
            notificationToast?.showToast('notice', 'Encounter resumed');
          }
        }
        lastPauseState = newPaused;
      });

      // Set up boss death listener
      const bossDeathUnlisten = await onBossDeath((event) => {
        notificationToast?.showToast('notice', `${event.payload.bossName} defeated!`);
      });

      // Set up scene change listener
      const sceneChangeUnlisten = await onSceneChange((event) => {
        console.log("Scene change event received:", event.payload);
        // notificationToast?.showToast('notice', `Scene changed to ${event.payload.sceneName}`);
      });

      console.log("Scene change listener set up");

      // Combine all unlisten functions
      unlisten = () => {
        playersUnlisten();
        resetUnlisten();
        encounterUnlisten();
        bossDeathUnlisten();
        sceneChangeUnlisten();
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

  // Save scroll position before navigating away
  beforeNavigate(({ from }) => {
    if (mainElement && from?.url.pathname) {
      scrollPositions.update(positions => ({
        ...positions,
        [from.url.pathname]: mainElement.scrollTop
      }));
    }
  });

  // Restore scroll position after navigation
  afterNavigate(({ to }) => {
    if (mainElement && to?.url.pathname) {
      const savedPosition = $scrollPositions[to.url.pathname];
      if (savedPosition !== undefined) {
        // Use requestAnimationFrame to ensure DOM is ready
        requestAnimationFrame(() => {
          if (mainElement) {
            mainElement.scrollTop = savedPosition;
          }
        });
      }
    }
  });

  onMount(() => {
    setupEventListeners();
    startReconnectCheck();

    return () => {
      if (reconnectInterval) clearInterval(reconnectInterval);
      if (unlisten) unlisten();
      cleanupStores();
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
<div class="flex h-screen flex-col text-xs text-white">
  <Header />
  <main bind:this={mainElement} class={cn("flex-1 overflow-y-auto px-2 py-2 gap-4", !SETTINGS.accessibility.state.transparency && "bg-neutral-900/40")}>
    <BossHealth />
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
