<script lang="ts">
  // import { commands } from "$lib/bindings";
  // import Button from "$lib/components/ui/button/button.svelte";
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import { settings } from "$lib/settings-store";

  const SETTINGS_CATEGORY = "misc";
  const MISC_SETTINGS = $derived(settings.state[SETTINGS_CATEGORY]);
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <!-- <Button onclick={commands.copySyncContainerData}>Dump SyncContainerData to clipboard</Button> -->
</Tabs.Content>
