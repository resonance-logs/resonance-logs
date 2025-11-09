<script lang="ts">
  /**
   * @file This is the layout for the main application window.
   * It sets up the top navigation and listens for navigation events.
   */
  import { setupShortcuts } from "./settings/shortcuts";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { SIDEBAR_ROUTES } from "./routes.svelte";
  import { getVersion } from "@tauri-apps/api/app";

  let { children } = $props();

  $effect.pre(() => {
    (async () => {
      await setupShortcuts();
    })();
  });

  const appWebview = getCurrentWebviewWindow();
  appWebview.listen<string>("navigate", (event) => {
    const route = event.payload;
    goto(route);
  });

  let currentPath = $derived(() => {
    const pathname = page.url.pathname;
    // Check for dynamic routes
    if (pathname.startsWith("/main/history/")) {
      return "/main/history";
    }
    return pathname;
  });
</script>

<div class="flex h-screen flex-col bg-background text-foreground p-3 gap-3">
  <!-- Top Navigation Bar -->
  <nav class="bg-card/50 rounded-lg border border-border/50 shrink-0">
    <div class="flex items-center justify-between px-4 py-2.5">
      <div class="flex items-center gap-1">
        {#each Object.entries(SIDEBAR_ROUTES) as [href, route] (route.label)}
          <a
            {href}
            class="flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all duration-200 h-9 {currentPath() === href
              ? 'bg-muted text-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground hover:bg-popover/50'}"
          >
            <route.icon class="w-4 h-4 shrink-0" />
            <span class="whitespace-nowrap">{route.label}</span>
          </a>
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
