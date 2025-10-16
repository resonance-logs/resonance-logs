<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import Accessibility from "./accessibility.svelte";
  import General from "./general.svelte";
  import Live from "./live.svelte";
  import Misc from "./misc.svelte";
  import Shortcuts from "./shortcuts.svelte";

  const settingsTabs = [
    { id: "general", label: "General" },
    { id: "accessibility", label: "Accessibility" },
    { id: "shortcuts", label: "Shortcuts" },
    { id: "live", label: "Live" },
    { id: "misc", label: "Misc" },
  ];
</script>

<Tabs.Root value="general">
  <Tabs.List>
    {#each settingsTabs as settingsTab (settingsTab.id)}
      <Tabs.Trigger value={settingsTab.id}>{settingsTab.label}</Tabs.Trigger>
    {/each}
  </Tabs.List>
  <General />
  <Accessibility />
  <Shortcuts />
  <Live />
  <Misc />
</Tabs.Root>
