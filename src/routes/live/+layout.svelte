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
  import { onPlayersUpdate, onResetEncounter, onEncounterUpdate, onBossDeath, onSceneChange, onPauseEncounter, onDungeonLogUpdate, onResetPlayerMetrics, resetPlayerMetrics } from "$lib/api";
  import { writable } from "svelte/store";
  import { beforeNavigate, afterNavigate } from "$app/navigation";

  // Store for pause state
  export const isPaused = writable(false);

  // Store for scroll positions
  const scrollPositions = writable<Record<string, number>>({});

  import { setDpsPlayers, setHealPlayers, setTankedPlayers, clearMeterData, cleanupStores, setLiveDungeonLog, clearLiveDungeonLog, injectDummyData } from "$lib/stores/live-meter-store.svelte";
  import HeaderCustom from "./header-custom.svelte";

  import NotificationToast from "./notification-toast.svelte";

  let { children } = $props();
  // let screenshotDiv: HTMLDivElement | undefined = $state();

  // transparency handled via polling effect below
  let transparencyInterval: number | null = null;

  let notificationToast: NotificationToast;
  let mainElement: HTMLElement | undefined = undefined;
  let unlisten: (() => void) | null = null;

  // Track the current active segment type for fake reset on segment switches
  let lastActiveSegmentType: 'boss' | 'trash' | null = null;
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
  // Track if component is destroyed to prevent callbacks from firing after unmount
  let isDestroyed = false;

  async function setupEventListeners() {
    if (isDestroyed || isReconnecting || listenersSetupInProgress) return;
    listenersSetupInProgress = true;

    // If listeners are already attached, skip setup to avoid duplicates.
    if (unlisten) {
      listenersSetupInProgress = false;
      return;
    }

    try {
      // Set up unified players listener
      const playersUnlisten = await onPlayersUpdate((event) => {
        if (isDestroyed) return;
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

      if (isDestroyed) {
        playersUnlisten();
        listenersSetupInProgress = false;
        return;
      }

      // Set up reset encounter listener
      const resetUnlisten = await onResetEncounter(() => {
        if (isDestroyed) return;
        clearMeterData();
        clearLiveDungeonLog();
        lastActiveSegmentType = null; // Reset segment tracking
        notificationToast?.showToast('notice', 'Server change detected, resetting log');
      });

      if (isDestroyed) {
        playersUnlisten();
        resetUnlisten();
        listenersSetupInProgress = false;
        return;
      }

      // Set up dungeon log listener
      const dungeonLogUnlisten = await onDungeonLogUpdate((event) => {
        if (isDestroyed) return;
        lastEventTime = Date.now();
        hadAnyEvent = true;

        // Check for active segment and detect segment type changes
        const dungeonLog = event.payload;
        const activeSegment = dungeonLog.segments.find(s => !s.endedAtMs);
        const currentSegmentType = activeSegment?.segmentType ?? null;

        // If segment type changed (trash -> boss or boss -> trash), perform fake reset
        if (currentSegmentType !== null &&
            lastActiveSegmentType !== null &&
            currentSegmentType !== lastActiveSegmentType) {

          // Call backend to reset player metrics (this will clear stores and emit reset event)
          // Fire and forget - we don't need to wait for the result
          resetPlayerMetrics().catch(e => {
            console.error('Failed to reset player metrics:', e);
          });


        }

        // Update last segment type
        lastActiveSegmentType = currentSegmentType;

        // Update the dungeon log store
        setLiveDungeonLog(dungeonLog);
      });

      if (isDestroyed) {
        playersUnlisten();
        resetUnlisten();
        dungeonLogUnlisten();
        listenersSetupInProgress = false;
        return;
      }

      // Set up encounter update listener (pause/resume)
      const encounterUnlisten = await onEncounterUpdate((event) => {
        if (isDestroyed) return;
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

      if (isDestroyed) {
        playersUnlisten();
        resetUnlisten();
        dungeonLogUnlisten();
        encounterUnlisten();
        listenersSetupInProgress = false;
        return;
      }

      // Set up boss death listener
      const bossDeathUnlisten = await onBossDeath((event) => {
        if (isDestroyed) return;
        // Treat boss death as a keep-alive
        lastEventTime = Date.now();
        hadAnyEvent = true;
        notificationToast?.showToast('notice', `${event.payload.bossName} defeated!`);
      });

      if (isDestroyed) {
        playersUnlisten();
        resetUnlisten();
        dungeonLogUnlisten();
        encounterUnlisten();
        bossDeathUnlisten();
        listenersSetupInProgress = false;
        return;
      }

      // Set up scene change listener
      const sceneChangeUnlisten = await onSceneChange((event) => {
        if (isDestroyed) return;
        // Treat scene change as a keep-alive
        lastEventTime = Date.now();
        hadAnyEvent = true;
        console.log("Scene change event received:", event.payload);
        // notificationToast?.showToast('notice', `Scene changed to ${event.payload.sceneName}`);
      });

      if (isDestroyed) {
        playersUnlisten();
        resetUnlisten();
        dungeonLogUnlisten();
        encounterUnlisten();
        bossDeathUnlisten();
        sceneChangeUnlisten();
        listenersSetupInProgress = false;
        return;
      }

      // Listen for explicit pause/resume events as a keep-alive as well
      const pauseUnlisten = await onPauseEncounter((event) => {
        if (isDestroyed) return;
        lastEventTime = Date.now();
        hadAnyEvent = true;
        isPaused.set(!!event.payload);
      });

      if (isDestroyed) {
        playersUnlisten();
        resetUnlisten();
        dungeonLogUnlisten();
        encounterUnlisten();
        bossDeathUnlisten();
        sceneChangeUnlisten();
        pauseUnlisten();
        listenersSetupInProgress = false;
        return;
      }

      console.log("Scene change listener set up");

      // Listen for reset-player-metrics events (fired on segment transitions)
      const resetPlayerMetricsUnlisten = await onResetPlayerMetrics((event) => {
        if (isDestroyed) return;
        // Clear just the meter/player stores without clearing the encounter log
        clearMeterData();
        // If the backend provided a segment name, show it; otherwise simple 'New Segment'
        const segName = event.payload?.segmentName ?? null;
        notificationToast?.showToast('notice', segName ? `New Segment: ${segName}` : 'New Segment');
      });

      if (isDestroyed) {
        playersUnlisten();
        resetUnlisten();
        dungeonLogUnlisten();
        encounterUnlisten();
        bossDeathUnlisten();
        sceneChangeUnlisten();
        pauseUnlisten();
        resetPlayerMetricsUnlisten();
        listenersSetupInProgress = false;
        return;
      }

      // Combine all unlisten functions
      unlisten = () => {
        try { playersUnlisten(); } catch {}
        try { resetUnlisten(); } catch {}
        try { encounterUnlisten(); } catch {}
        try { bossDeathUnlisten(); } catch {}
        try { sceneChangeUnlisten(); } catch {}
        try { pauseUnlisten(); } catch {}
        try { dungeonLogUnlisten(); } catch {}
        try { resetPlayerMetricsUnlisten(); } catch {}
      };

      console.log("Event listeners set up for live meter data");

      listenersSetupInProgress = false;
    } catch (e) {
      console.error("Failed to set up event listeners:", e);
      listenersSetupInProgress = false;
      if (isDestroyed) return;
      isReconnecting = true;
      setTimeout(() => {
        isReconnecting = false;
        if (!isDestroyed) setupEventListeners();
      }, reconnectDelay);
      // increase backoff cap at ~10s
      reconnectDelay = Math.min(reconnectDelay * 2, 10_000);
    }
  }

  function startReconnectCheck() {
    reconnectInterval = setInterval(() => {
      if (isDestroyed) return;
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
    isDestroyed = false;
    setupEventListeners();
    startReconnectCheck();

    // When the window regains focus or visibility, proactively recheck listeners
    const onFocus = () => {
      if (isDestroyed) return;
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
    const onVisibilityChange = () => {
      if (document.visibilityState === "visible") onFocus();
    };
    document.addEventListener("visibilitychange", onVisibilityChange);

    // Poll settings for transparency changes and apply CSS variables / body background
    transparencyInterval = window.setInterval(() => {
      if (isDestroyed) return;
      try {
        const enabled = !!SETTINGS.accessibility.state.transparency;
        const percent = Number(SETTINGS.accessibility.state.transparentOpacityPercent ?? 2) || 2;
        const opacity = String(percent / 100);
        
        // Apply background image if enabled (for custom theme)
        const bgImageEnabled = SETTINGS.accessibility.state.backgroundImageEnabled;
        const bgImage = SETTINGS.accessibility.state.backgroundImage;
        const bgMode = SETTINGS.accessibility.state.backgroundImageMode || 'cover';
        const bgContainColor = SETTINGS.accessibility.state.backgroundImageContainColor || 'rgba(0, 0, 0, 1)';
        const isCustomTheme = SETTINGS.accessibility.state.theme === 'custom';
        
        if (isCustomTheme && bgImageEnabled && bgImage) {
          document.body.style.backgroundImage = `url('${bgImage}')`;
          document.body.style.backgroundSize = bgMode;
          document.body.style.backgroundPosition = 'center';
          document.body.style.backgroundRepeat = 'no-repeat';
          if (bgMode === 'contain') {
            document.body.style.backgroundColor = bgContainColor;
          } else {
            document.body.style.backgroundColor = '';
          }
          document.documentElement.classList.remove('transparent-mode');
        } else if (enabled) {
          // Add root-level class so our CSS rules apply
          document.documentElement.classList.add('transparent-mode');
          document.documentElement.style.setProperty('--bg-opacity', opacity);
          // Make the page background fully transparent so the window shows through
          document.body.style.background = 'transparent';
          document.body.style.backgroundImage = '';
        } else {
          document.documentElement.classList.remove('transparent-mode');
          document.body.style.background = '';
          document.body.style.backgroundImage = '';
          document.body.style.backgroundColor = '';
        }
        
        // Apply custom fonts if enabled
        const sansEnabled = SETTINGS.accessibility.state.customFontSansEnabled;
        const sansName = SETTINGS.accessibility.state.customFontSansName;
        const sansUrl = SETTINGS.accessibility.state.customFontSansUrl;
        const monoEnabled = SETTINGS.accessibility.state.customFontMonoEnabled;
        const monoName = SETTINGS.accessibility.state.customFontMonoName;
        const monoUrl = SETTINGS.accessibility.state.customFontMonoUrl;
        
        // Load custom fonts if URLs are set (need to register font faces)
        if (sansEnabled && sansName && sansUrl) {
          // Check if font is already registered
          if (!document.fonts.check(`12px "${sansName}"`)) {
            const fontFace = new FontFace(sansName, `url(${sansUrl})`);
            fontFace.load().then((loadedFace) => {
              document.fonts.add(loadedFace);
            }).catch(() => {});
          }
          document.documentElement.style.setProperty('--font-sans', `"${sansName}", sans-serif`);
        } else {
          document.documentElement.style.setProperty('--font-sans', '"Inter Variable", sans-serif');
        }
        
        if (monoEnabled && monoName && monoUrl) {
          // Check if font is already registered
          if (!document.fonts.check(`12px "${monoName}"`)) {
            const fontFace = new FontFace(monoName, `url(${monoUrl})`);
            fontFace.load().then((loadedFace) => {
              document.fonts.add(loadedFace);
            }).catch(() => {});
          }
          document.documentElement.style.setProperty('--font-mono', `"${monoName}", monospace`);
        } else {
          document.documentElement.style.setProperty('--font-mono', '"Geist Mono Variable", monospace');
        }
      } catch (e) {
        // ignore
      }
    }, 200);
    return () => {
      isDestroyed = true;
      if (reconnectInterval) clearInterval(reconnectInterval);
      if (transparencyInterval) clearInterval(transparencyInterval);
      if (unlisten) unlisten();
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVisibilityChange);
      cleanupStores();
    };
  });

  // Watch for dummy data toggle
  $effect(() => {
    if (SETTINGS.live.general.state.useDummyData) {
      injectDummyData();
    } else {
      cleanupStores()
    }
  });

  // Blur feature removed; effect removed.
</script>

<!-- flex flex-col min-h-screen → makes the page stretch full height and stack header, body, and footer. -->
<!-- flex-1 on <main> → makes the body expand to fill leftover space, pushing the footer down. -->
  <div 
    class="flex h-screen flex-col bg-background text-[13px] text-foreground rounded-xl shadow-[0_10px_30px_-10px_rgba(0,0,0,0.6)]" 
    style="padding: {SETTINGS.live.headerCustomization.state.windowPadding}px"
    data-tauri-drag-region
  >
    <HeaderCustom />
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
