<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSelect from "./settings-select.svelte";
  import SettingsSlider from "./settings-slider.svelte";
  import SettingsSwitch from "./settings-switch.svelte";
  import { SETTINGS, AVAILABLE_THEMES, DEFAULT_CLASS_COLORS, DEFAULT_FONT_SIZES, FONT_SIZE_LABELS } from "$lib/settings-store";
  import { setClickthrough, CLASS_NAMES, getClassColorRaw } from "$lib/utils.svelte";

  const SETTINGS_CATEGORY = "themes";

  const FONT_SIZE_KEYS = Object.keys(DEFAULT_FONT_SIZES) as (keyof typeof DEFAULT_FONT_SIZES)[];

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

  function updateFontSize(key: string, size: number) {
    SETTINGS.accessibility.state.fontSizes = {
      ...SETTINGS.accessibility.state.fontSizes,
      [key]: size,
    };
  }

  function resetFontSizes() {
    SETTINGS.accessibility.state.fontSizes = { ...DEFAULT_FONT_SIZES };
  }

  function getFontSize(key: string): number {
    const fontSizes = SETTINGS.accessibility.state.fontSizes ?? DEFAULT_FONT_SIZES;
    return fontSizes[key as keyof typeof DEFAULT_FONT_SIZES] ?? DEFAULT_FONT_SIZES[key as keyof typeof DEFAULT_FONT_SIZES];
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
      <div class="mt-1">
        <SettingsSwitch
          bind:checked={SETTINGS.accessibility.state.clickthrough}
          label="Clickthrough Mode"
          description={SETTINGS.accessibility.state.clickthrough ? 'Clickthrough Enabled - Mouse clicks pass through window' : 'Enable Clickthrough Mode'}
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

    <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-base font-semibold text-foreground">Font Sizes</h2>
          <p class="text-xs text-muted-foreground mt-1">Customize font sizes throughout the app (in pixels).</p>
        </div>
        <button
          onclick={resetFontSizes}
          class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors"
        >
          Reset to Defaults
        </button>
      </div>
      <div class="grid grid-cols-1 gap-2 mt-2">
        {#each FONT_SIZE_KEYS as key}
          <div class="flex items-center justify-between py-2 px-3 rounded-md hover:bg-popover/50 transition-colors">
            <div class="flex flex-col">
              <span class="text-sm font-medium text-foreground">{FONT_SIZE_LABELS[key]}</span>
              <span class="text-xs text-muted-foreground">Used for {key === 'xs' ? 'labels and hints' : key === 'sm' ? 'secondary text' : key === 'base' ? 'default text' : key === 'lg' ? 'headings' : 'titles'}</span>
            </div>
            <div class="flex items-center gap-2">
              <input
                type="range"
                min="8"
                max="32"
                step="1"
                value={getFontSize(key)}
                oninput={(e) => updateFontSize(key, parseInt(e.currentTarget.value))}
                class="w-24 h-2 rounded-lg appearance-none cursor-pointer bg-muted"
              />
              <input
                type="number"
                min="8"
                max="32"
                value={getFontSize(key)}
                oninput={(e) => updateFontSize(key, parseInt(e.currentTarget.value) || DEFAULT_FONT_SIZES[key])}
                class="w-14 px-2 py-1 text-sm text-center bg-popover border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary/50"
              />
              <span class="text-xs text-muted-foreground w-4">px</span>
            </div>
          </div>
        {/each}
      </div>
    </div>

  </div>
</Tabs.Content>
