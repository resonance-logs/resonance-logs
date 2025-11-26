<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSelect from "./settings-select.svelte";
  import SettingsSlider from "./settings-slider.svelte";
  import SettingsSwitch from "./settings-switch.svelte";
  import { SETTINGS, AVAILABLE_THEMES, DEFAULT_CLASS_COLORS } from "$lib/settings-store";
  import { setClickthrough, CLASS_NAMES, getClassColorRaw } from "$lib/utils.svelte";

  const SETTINGS_CATEGORY = "themes";

  // Sync clickthrough state with the setting
  $effect(() => {
    setClickthrough(SETTINGS.accessibility.state.clickthrough);
  });

  function updateClassColor(className: string, color: string) {
    SETTINGS.accessibility.state.classColors = {
      ...SETTINGS.accessibility.state.classColors,
      [className]: color,
    };
  }

  function resetClassColors() {
    SETTINGS.accessibility.state.classColors = { ...DEFAULT_CLASS_COLORS };
  }
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
        <SettingsSwitch
          bind:checked={SETTINGS.accessibility.state.clickthrough}
          label="Clickthrough Mode"
          description={SETTINGS.accessibility.state.clickthrough ? 'Clickthrough Enabled - Mouse clicks pass through window' : 'Enable Clickthrough Mode'}
        />
      </div>

      <div class="mt-1">
        <SettingsSwitch
          bind:checked={SETTINGS.live.general.state.useDummyData}
          label="Use Dummy Data"
          description="Inject dummy player data into the live meter for testing and preview purposes"
        />
      </div>
    </div>

    <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-base font-semibold text-foreground">Class Colors</h2>
          <p class="text-xs text-muted-foreground mt-1">Customize the color for each class displayed in the meter.</p>
        </div>
        <button
          onclick={resetClassColors}
          class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors"
        >
          Reset to Defaults
        </button>
      </div>
      <div class="grid grid-cols-2 gap-2 mt-2">
        {#each CLASS_NAMES as className}
          <label class="flex items-center gap-3 py-2 px-3 rounded-md hover:bg-popover/50 transition-colors">
            <input
              type="color"
              value={getClassColorRaw(className)}
              oninput={(e) => updateClassColor(className, e.currentTarget.value)}
              class="w-8 h-8 rounded cursor-pointer border border-border/50"
            />
            <span class="text-sm font-medium text-foreground">{className}</span>
          </label>
        {/each}
      </div>
    </div>
  </div>
</Tabs.Content>
