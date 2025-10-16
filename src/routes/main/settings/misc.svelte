<script lang="ts">
  import { commands } from "$lib/bindings";
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { SETTINGS } from "$lib/settings-store";
  import SettingsSwitch from "./settings-switch.svelte";

  const SETTINGS_CATEGORY = "misc";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <Button onclick={commands.copySyncContainerData}>Dump SyncContainerData to clipboard</Button>
  <SettingsSwitch bind:checked={SETTINGS.misc.state.testingMode} label="Testing Mode" description="Enable UI Testing. Only works with DPS/Heal Player/Skills. Once you turn it off, make sure to reload the window." />
</Tabs.Content>
