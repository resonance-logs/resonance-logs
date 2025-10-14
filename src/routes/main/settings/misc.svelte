<script lang="ts">
  import { commands } from "$lib/bindings";
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Tabs from "$lib/components/ui/tabs/index.js";

  const SETTINGS_CATEGORY = "misc";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <Button
    onclick={commands.copySyncContainerData}
  >
    Dump SyncContainerData to clipboard
  </Button>
</Tabs.Content>
