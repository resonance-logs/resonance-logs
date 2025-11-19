<script lang="ts">
  /**
   * @file This is the layout for the live meter.
   * It sets up event listeners for live data, manages the pause state,
   * and handles scroll position restoration.
   *
   * It also displays the header, footer, boss health, and notification toasts.
   *
   * @packageDocumentation
   */
  import { onMount } from "svelte";
  import { SETTINGS } from "$lib/settings-store";
  import { onPlayersUpdate, onResetEncounter, onEncounterUpdate, onBossDeath, onSceneChange, onPauseEncounter } from "$lib/api";
  import { writable } from "svelte/store";
  import { beforeNavigate, afterNavigate } from "$app/navigation";

  // Store for pause state
  export const isPaused = writable(false);

  // Store for scroll positions
  const scrollPositions = writable<Record<string, number>>({});

  import { setDpsPlayers, setHealPlayers, setTankedPlayers, clearMeterData, cleanupStores } from "$lib/stores/live-meter-store.svelte";
  import Header from "./header.svelte";
  import HeaderOneRow from "./header-one-row.svelte"

  import NotificationToast from "./notification-toast.svelte";

  let { children } = $props();
  // let screenshotDiv: HTMLDivElement | undefined = $state();

  // transparency handled via polling effect below
  let transparencyInterval: number | null = null;

  let notificationToast: NotificationToast;
  let mainElement: HTMLElement | undefined = undefined;
  let unlisten: (() => void) | null = null;
  // Prevent concurrent setupEventListeners runs which can attach duplicate listeners
  let listenersSetupInProgress = false;
  let lastEventTime = Date.now();
  let hadAnyEvent = false; // becomes true after the first live event arrives
  // Persist last known pause state across listener reconnections so we don't
  // show a spurious "Encounter resumed" toast every time listeners are
  // re-attached (e.g. on window focus/visibility change).
  let lastPauseState: boolean | null = null;
  let reconnectInterval: ReturnType<typeof setInterval> | null = null;
  let isReconnecting = false;
  let reconnectDelay = 1000; // exponential backoff base
  const DISCONNECT_THRESHOLD = 5000;

  async function setupEventListeners() {
    if (isReconnecting || listenersSetupInProgress) return;
    listenersSetupInProgress = true;

    // If listeners are already attached, skip setup to avoid duplicates.
    if (unlisten) {
      listenersSetupInProgress = false;
      return;
    }

    try {
      // Set up unified players listener
      const playersUnlisten = await onPlayersUpdate((event) => {
        // console.log("players websocket", event.payload)
        lastEventTime = Date.now();
        hadAnyEvent = true;

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
      const encounterUnlisten = await onEncounterUpdate((event) => {
        // Treat encounter updates as keep-alive too so reconnect logic doesn't fire
        lastEventTime = Date.now();
        hadAnyEvent = true;
        const newPaused = event.payload.isPaused;
        const elapsedMs = event.payload.headerInfo.elapsedMs;
        // update the store regardless
        isPaused.set(newPaused);
        // only show a toast if the pause state actually changed AND we've started receiving combat data
        // Note: do NOT show a toast on the initial listener attach (lastPauseState === null)
        // to avoid spurious "Encounter resumed" messages when reattaching listeners
        if (elapsedMs > 0 && lastPauseState !== null && lastPauseState !== newPaused) {
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
        // Treat boss death as a keep-alive
        lastEventTime = Date.now();
        hadAnyEvent = true;
        notificationToast?.showToast('notice', `${event.payload.bossName} defeated!`);
      });

      // Set up scene change listener
      const sceneChangeUnlisten = await onSceneChange((event) => {
        // Treat scene change as a keep-alive
        lastEventTime = Date.now();
        hadAnyEvent = true;
        console.log("Scene change event received:", event.payload);
        // notificationToast?.showToast('notice', `Scene changed to ${event.payload.sceneName}`);
      });

      // Listen for explicit pause/resume events as a keep-alive as well
      const pauseUnlisten = await onPauseEncounter((event) => {
        lastEventTime = Date.now();
        hadAnyEvent = true;
        isPaused.set(!!event.payload);
      });

      console.log("Scene change listener set up");

      // Combine all unlisten functions
      unlisten = () => {
        try { playersUnlisten(); } catch {}
        try { resetUnlisten(); } catch {}
        try { encounterUnlisten(); } catch {}
        try { bossDeathUnlisten(); } catch {}
        try { sceneChangeUnlisten(); } catch {}
        try { pauseUnlisten(); } catch {}
      };

      console.log("Event listeners set up for live meter data");
      listenersSetupInProgress = false;
    } catch (e) {
      console.error("Failed to set up event listeners:", e);
      listenersSetupInProgress = false;
      isReconnecting = true;
      setTimeout(() => {
        isReconnecting = false;
        setupEventListeners();
      }, reconnectDelay);
      // increase backoff cap at ~10s
      reconnectDelay = Math.min(reconnectDelay * 2, 10_000);
    }
  }

  function startReconnectCheck() {
    reconnectInterval = setInterval(() => {
      const now = Date.now();
      if (hadAnyEvent && now - lastEventTime > DISCONNECT_THRESHOLD) {
        console.warn("Live event stream disconnected, attempting reconnection");
        if (unlisten) {
          unlisten();
          unlisten = null;
        }
        // reset timer to avoid tight loop spam
        lastEventTime = now;
        setupEventListeners();
        // backoff after each timer-triggered reconnect
        reconnectDelay = Math.min(reconnectDelay * 2, 10_000);
      }
    }, 1000);
  }

  // Save scroll position before navigating away
  beforeNavigate(({ from }) => {
    if (mainElement && from?.url.pathname) {
      scrollPositions.update(positions => ({
        ...positions,
        [from.url.pathname]: mainElement!.scrollTop
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

    // When the window regains focus or visibility, proactively recheck listeners
    const onFocus = () => {
      // Attempt a lightweight refresh of listeners to recover from any suspended state
      if (!isReconnecting) {
        if (unlisten) {
          unlisten();
          unlisten = null;
        }
        reconnectDelay = 1000;
        setupEventListeners();
      }
    };
    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", () => {
      if (document.visibilityState === "visible") onFocus();
    });

    // Poll settings for transparency changes and apply CSS variables / body background
    transparencyInterval = window.setInterval(() => {
      try {
        const enabled = !!SETTINGS.accessibility.state.transparency;
        const percent = Number(SETTINGS.accessibility.state.transparentOpacityPercent ?? 2) || 2;
        const opacity = String(percent / 100);
        if (enabled) {
          // Add root-level class so our CSS rules apply
          document.documentElement.classList.add('transparent-mode');
          document.documentElement.style.setProperty('--bg-opacity', opacity);
          // Make the page background fully transparent so the window shows through
          document.body.style.background = 'transparent';
        } else {
          document.documentElement.classList.remove('transparent-mode');
          document.body.style.background = '';
        }
      } catch (e) {
        // ignore
      }
    }, 200);
    return () => {
      if (reconnectInterval) clearInterval(reconnectInterval);
      if (transparencyInterval) clearInterval(transparencyInterval);
      if (unlisten) unlisten();
      window.removeEventListener("focus", onFocus);
      // visibilitychange listener is anonymous; it's fine to leave or we can no-op
      cleanupStores();
    };
  });

  // Blur feature removed; effect removed.
</script>

<!-- flex flex-col min-h-screen → makes the page stretch full height and stack header, body, and footer. -->
<!-- flex-1 on <main> → makes the body expand to fill leftover space, pushing the footer down. -->
  <div class="flex h-screen flex-col bg-background text-[13px] text-foreground p-3 rounded-xl shadow-[0_10px_30px_-10px_rgba(0,0,0,0.6)]" data-tauri-drag-region>
    {#if SETTINGS.accessibility.state.condenseHeader == "full"}
      <Header />
    {:else if SETTINGS.accessibility.state.condenseHeader == "one row"}
      <HeaderOneRow/>
    {/if}
    <main
    bind:this={mainElement}
    class="flex-1 overflow-y-auto gap-4 rounded-lg bg-card/20 border border-border/40"
    >
    {@render children()}
  </main>
  <!-- Footer removed; navigation and version moved into Header -->
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
