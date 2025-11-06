<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import Accessibility from "./accessibility.svelte";
  import Live from "./live.svelte";
  import History from "./history.svelte";
  import Misc from "./misc.svelte";
  import Shortcuts from "./shortcuts.svelte";

  const settingsTabs = [
    { id: "live", label: "Live" },
    { id: "history", label: "Past Encounters" },
    { id: "accessibility", label: "Accessibility" },
    { id: "shortcuts", label: "Shortcuts" },
    { id: "misc", label: "Misc" },
  ];
  // Track the active tab so we can lazy-mount tab content. The Tabs implementation
  // used here will keep all children mounted by default which causes every
  // settings tab to initialize on page load. That in turn triggers backend
  // side-effects (e.g. setBossOnlyDps) from every tab and can stall the live
  // emitter. We render only the currently active tab component to avoid that.
  let activeTab = $state('live');
</script>

<Tabs.Root bind:value={activeTab}>
  <Tabs.List>
    {#each settingsTabs as settingsTab (settingsTab.id)}
      <Tabs.Trigger value={settingsTab.id}>{settingsTab.label}</Tabs.Trigger>
    {/each}
  </Tabs.List>

  {#if activeTab === 'live'}
    <Live />
  {:else if activeTab === 'history'}
    <History />
  {:else if activeTab === 'accessibility'}
    <Accessibility />
  {:else if activeTab === 'shortcuts'}
    <Shortcuts />
  {:else}
    <Misc />
  {/if}
</Tabs.Root>
