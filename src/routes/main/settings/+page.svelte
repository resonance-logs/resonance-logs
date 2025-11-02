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
</script>

<Tabs.Root value="live">
  <Tabs.List>
    {#each settingsTabs as settingsTab (settingsTab.id)}
      <Tabs.Trigger value={settingsTab.id}>{settingsTab.label}</Tabs.Trigger>
    {/each}
  </Tabs.List>
  <Live />
  <History />
  <Accessibility />
  <Shortcuts />
  <Misc />
</Tabs.Root>
