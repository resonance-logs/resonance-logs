<script lang="ts">
  /**
   * @file This component displays the sidebar of the main application window.
   * It includes links to the different pages of the application.
   */
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  import { getVersion } from "@tauri-apps/api/app";
  import { SIDEBAR_ROUTES } from "./routes.svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";

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
</script>

<Sidebar.Root>
  <Sidebar.Header>Resonance Logs</Sidebar.Header>
  <Sidebar.Separator />
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each Object.entries(SIDEBAR_ROUTES) as [href, route] (route.label)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton>
                {#snippet child({ props })}
                  {#if route.externalUrl}
                    <button type="button" {...props} on:click={() => openExternalUrl(route.externalUrl)}>
                      <route.icon />
                      <span class="inline-flex items-center gap-1">
                        {route.label}
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                        </svg>
                      </span>
                    </button>
                  {:else}
                    <a {href} {...props}>
                      <route.icon />
                      <span>{route.label}</span>
                    </a>
                  {/if}
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
  <Sidebar.Footer><span>v{#await getVersion()}X.Y.Z{:then version}{version}{/await}</span></Sidebar.Footer>
</Sidebar.Root>
