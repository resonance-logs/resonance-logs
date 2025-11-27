<script lang="ts">
  /**
   * @file This is the layout for the main application window.
   * It sets up the top navigation and listens for navigation events.
   */
  import { setupShortcuts } from "./settings/shortcuts";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { SIDEBAR_ROUTES } from "./routes.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import ChangelogModal from '$lib/components/ChangelogModal.svelte';
  import { SETTINGS } from '$lib/settings-store';
  import { onMount } from 'svelte';

  let { children } = $props();

  $effect.pre(() => {
    (async () => {
      await setupShortcuts();
    })();
  });

  // Navigation listener is set up in onMount and properly cleaned up
  let navigateUnlisten: (() => void) | null = null;
  let transparencyInterval: number | null = null;

  async function openExternalUrl(url: string) {
    try {
      await openUrl(url);
    } catch (err) {
      console.error("Failed to open external URL:", url, err);
      if (typeof window !== "undefined") {
        window.open(url, "_blank");
      }
    }
  }

  let currentPath = $derived(() => {
    const pathname = page.url.pathname;
    // Check for dynamic routes
    if (pathname.startsWith("/main/history/")) {
      return "/main/history";
    }
    return pathname;
  });

  let showChangelog = $state(false);
  let currentVersion = $state('');

  onMount(() => {
    // Set up navigation listener
    const appWebview = getCurrentWebviewWindow();
    appWebview.listen<string>("navigate", (event) => {
      const route = event.payload;
      goto(route);
    }).then((unlisten) => {
      navigateUnlisten = unlisten;
    });

    // Get app version and check changelog
    getVersion().then((v) => {
      currentVersion = v;
      // Compare persisted last-seen version with current app version
      if ((SETTINGS.appVersion.state as any).value !== v) {
        showChangelog = true;
      }
    }).catch((err) => {
      console.error('Failed to get app version', err);
    });

    // Poll settings for transparency changes and apply CSS variables / body background
    transparencyInterval = window.setInterval(() => {
      try {
        
        // Apply background image if enabled and custom theme
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

    // Cleanup on unmount
    return () => {
      if (navigateUnlisten) {
        navigateUnlisten();
        navigateUnlisten = null;
      }
      if (transparencyInterval) {
        clearInterval(transparencyInterval);
      }
    };
  });

  function handleClose() {
    // mark changelog as seen for this version
    try {
      (SETTINGS.appVersion.state as any).value = currentVersion;
    } catch (e) {
      console.error('Failed to set appVersion setting', e);
    }
    showChangelog = false;
  }
</script>

<div class="flex h-screen flex-col bg-background-main text-foreground p-3 gap-3">
  <!-- Top Navigation Bar -->
  <nav class="bg-card/50 rounded-lg border border-border/50 shrink-0">
    <div class="flex items-center justify-between px-4 py-2.5">
      <div class="flex items-center gap-1">
        {#each Object.entries(SIDEBAR_ROUTES) as [href, route] (route.label)}
          {#if (route as any).externalUrl}
            <button
              type="button"
              class="flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all duration-200 h-9 text-muted-foreground hover:text-foreground hover:bg-popover/50"
              title={`Open ${route.label} in browser`}
              onclick={() => openExternalUrl((route as any).externalUrl)}
            >
              <route.icon class="w-4 h-4 shrink-0" />
              <span class="whitespace-nowrap flex items-center gap-1">
                {route.label}
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                </svg>
              </span>
            </button>
          {:else}
            <a
              {href}
              class="flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all duration-200 h-9 {currentPath() === href
                ? 'bg-muted text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground hover:bg-popover/50'}"
            >
              <route.icon class="w-4 h-4 shrink-0" />
              <span class="whitespace-nowrap">{route.label}</span>
            </a>
          {/if}
        {/each}
      </div>
      <div class="text-xs text-muted-foreground shrink-0">
        v{#await getVersion()}...{:then version}{version}{/await}
      </div>
    </div>
  </nav>

  <!-- Main Content -->
  <main class="flex-1 overflow-y-auto rounded-lg bg-card/30 border border-border/50">
    <div class="p-6">
      {@render children()}
    </div>
  </main>

  {#if showChangelog}
    <ChangelogModal on:close={handleClose} />
  {/if}
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
