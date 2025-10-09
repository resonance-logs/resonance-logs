<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { settings } from "$lib/settings-store";
  import SettingsSwitch from "./settings-switch.svelte";

  const SETTINGS_CATEGORY = "accessibility"
  const ACCESSIBILITY_SETTINGS = $derived(settings.state[SETTINGS_CATEGORY]);
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <SettingsSwitch bind:checked={ACCESSIBILITY_SETTINGS["blur"]} label="Blur Meter Background" description="Adds background blur effect to live meter. Bad performance when resizing/dragging the window on Windows 11 build 22621+ unless you need to disable Transparency effects in Windows accessibility settings."/>
</Tabs.Content>
