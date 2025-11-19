<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSelect from "./settings-select.svelte";
  import SettingsSlider from "./settings-slider.svelte";
  import SettingsSwitch from "./settings-switch.svelte";
  import { SETTINGS, AVAILABLE_THEMES } from "$lib/settings-store";

  const SETTINGS_CATEGORY = "themes";

  // bind directly to `SETTINGS.accessibility.state.transparency` via SettingsSwitch
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <div class="space-y-3">
    <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
      <div>
        <h2 class="text-base font-semibold text-foreground">Color Themes</h2>
        <p class="text-xs text-muted-foreground mt-1">Pick a visual theme. Changes apply instantly and persist.</p>
      </div>
      <SettingsSelect
        label="Theme"
        description="Choose one of the built-in themes"
        bind:selected={SETTINGS.accessibility.state["theme"]}
        values={AVAILABLE_THEMES}
      />
    </div>

    <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
      <div>
        <h2 class="text-base font-semibold text-foreground">Transparency Mode</h2>
        <p class="text-xs text-muted-foreground mt-1">Enable transparent meter surfaces and adjust opacity.</p>
      </div>
      <div class="mt-1 space-y-2">
        <SettingsSwitch
          bind:checked={SETTINGS.accessibility.state.transparency}
          label="Transparent Mode"
          description={SETTINGS.accessibility.state.transparency ? 'Transparent Mode Enabled' : 'Enable Transparent Mode'}
        />
        <SettingsSlider bind:value={SETTINGS.accessibility.state["transparentOpacityPercent"]} min={0} max={100} step={1} label="Transparency Opacity" description="Lower values make the meter more see-through. 0% is fully transparent." unit="%" />
      </div>
    </div>

    <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
      <div>
        <h2 class="text-base font-semibold text-foreground">Live Meter Display Settings</h2>
      </div>
      <div class="mt-1">
        <SettingsSelect
          label="Header Size"
          description="Choose compactness for header rows"
          bind:selected={SETTINGS.accessibility.state.condenseHeader}
          values={["full", "one row", "none",]}
        />
      </div>
      <div class="mt-1">
        <SettingsSelect
          label="Density"
          description="Choose compactness for live meter"
          bind:selected={SETTINGS.accessibility.state.density}
          values={["comfortable","medium","compact"]}
        />
      </div>
    </div>

  </div>
</Tabs.Content>
