<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { SETTINGS } from "$lib/settings-store";
  import SettingsSwitch from "./settings-switch.svelte";
  // SettingsSelect not used in this tab currently (theme selection moved). Removed import.

  const SETTINGS_CATEGORY = "accessibility";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <div class="space-y-3">
    <!-- Theme selection moved to dedicated Themes tab -->
    <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-1">
      <h2 class="text-base font-semibold text-foreground mb-3">Visual Effects</h2>
      <SettingsSwitch bind:checked={SETTINGS.accessibility.state["blur"]} label="Blur Meter Background" description="Adds background blur effect to live meter (Windows 10 only). Bad performance when resizing/dragging the window on Windows 11 build 22621+ unless you to disable Transparency effects in Windows accessibility SETTINGS." />
      <SettingsSwitch bind:checked={SETTINGS.accessibility.state["transparency"]} label="Transparent Meter" description="Toggle transparent background for live meter." />
    </div>
  </div>
</Tabs.Content>
