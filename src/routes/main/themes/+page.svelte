<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSelect from "../settings/settings-select.svelte";
  import SettingsSlider from "../settings/settings-slider.svelte";
  import SettingsSwitch from "../settings/settings-switch.svelte";
  import SettingsColor from "../settings/settings-color.svelte";
  import SettingsColorAlpha from "../settings/settings-color-alpha.svelte";
  import SettingsFilePicker from "../settings/settings-file-picker.svelte";
  import { SETTINGS, DEFAULT_CLASS_COLORS, DEFAULT_CLASS_SPEC_COLORS, CLASS_SPEC_NAMES, DEFAULT_CUSTOM_THEME_COLORS, CUSTOM_THEME_COLOR_LABELS } from "$lib/settings-store";
  import { setClickthrough, CLASS_NAMES, getClassColorRaw } from "$lib/utils.svelte";
  import { setBossOnlyDps, setDungeonSegmentsEnabled } from "$lib/api";
  import { onMount } from 'svelte';
  import ChevronDown from "virtual:icons/lucide/chevron-down";

  const themesTabs = [
    { id: "general", label: "General" },
    { id: "live", label: "Live" },
    { id: "presets", label: "Presets" },
  ];

  // === COLOR THEME PRESETS (matching CSS data-theme selectors) ===
  const COLOR_PRESETS: Record<string, { name: string; description: string; theme: string; preview: { bg: string; primary: string; accent: string; fg: string } }> = {
    dark: {
      name: "Dark",
      description: "Clean dark theme with neutral grays",
      theme: 'dark',
      preview: { bg: '#212121', primary: '#a6a6a6', accent: '#525252', fg: '#e2e2e2' }
    },
    light: {
      name: "Light",
      description: "Bright theme for daylight use",
      theme: 'light',
      preview: { bg: '#fbfbf9', primary: '#5b7fc7', accent: '#d4a84a', fg: '#2a2e40' }
    },
    pink: {
      name: "Pink UwU",
      description: "Cute pastel pink aesthetic",
      theme: 'pink',
      preview: { bg: '#F8E8EE', primary: '#F2BED1', accent: '#F2BED1', fg: '#582F3B' }
    },
    green: {
      name: "Green Pastel",
      description: "Soft green nature tones",
      theme: 'green',
      preview: { bg: '#e0f0e0', primary: '#6fbf6f', accent: '#7fcf8f', fg: '#1a2a1a' }
    },
    matcha: {
      name: "Matcha",
      description: "Earthy green tea vibes",
      theme: 'matcha',
      preview: { bg: '#d8e8d0', primary: '#5a9f5a', accent: '#6ab06a', fg: '#283828' }
    },
    rainbow: {
      name: "Pastel Rainbow",
      description: "Colorful gradient background",
      theme: 'rainbow',
      preview: { bg: 'linear-gradient(120deg,#ffe5ec,#e0f7fa,#f3e8ff,#e9fbd5)', primary: '#b87fd0', accent: '#d09050', fg: '#383848' }
    },
  };

  // === SIZE PRESETS ===
  const SIZE_PRESETS: Record<string, { name: string; description: string; table: Record<string, number | string | boolean>; header: Record<string, number | boolean> }> = {
    compact: {
      name: "Compact",
      description: "Minimal - no padding, no header",
      table: {
        playerRowHeight: 20,
        playerFontSize: 10,
        playerIconSize: 14,
        showTableHeader: false,
        tableHeaderHeight: 18,
        tableHeaderFontSize: 8,
        abbreviatedFontSize: 7,
        skillRowHeight: 18,
        skillFontSize: 9,
        skillIconSize: 12,
        skillShowHeader: false,
        skillHeaderHeight: 16,
        skillHeaderFontSize: 7,
        skillAbbreviatedFontSize: 6,
        rowGlowMode: 'gradient-underline',
        skillRowGlowMode: 'gradient-underline',
        rowGlowOpacity: 0.5,
        skillRowGlowOpacity: 0.5,
        rowBorderRadius: 0,
        skillRowBorderRadius: 0,
      },
      header: {
        windowPadding: 0,
        headerPadding: 4,
        timerLabelFontSize: 9,
        timerFontSize: 12,
        sceneNameFontSize: 10,
        segmentFontSize: 9,
        resetButtonSize: 14,
        resetButtonPadding: 4,
        pauseButtonSize: 14,
        pauseButtonPadding: 4,
        bossOnlyButtonSize: 14,
        bossOnlyButtonPadding: 4,
        settingsButtonSize: 14,
        settingsButtonPadding: 4,
        minimizeButtonSize: 14,
        minimizeButtonPadding: 4,
        totalDamageLabelFontSize: 9,
        totalDamageValueFontSize: 12,
        totalDpsLabelFontSize: 9,
        totalDpsValueFontSize: 12,
        bossHealthLabelFontSize: 9,
        bossHealthNameFontSize: 10,
        bossHealthValueFontSize: 10,
        bossHealthPercentFontSize: 10,
        navTabFontSize: 8,
        navTabPaddingX: 6,
        navTabPaddingY: 3,
      }
    },
    small: {
      name: "Small",
      description: "Compact layout for more rows",
      table: {
        playerRowHeight: 22,
        playerFontSize: 11,
        playerIconSize: 16,
        showTableHeader: true,
        tableHeaderHeight: 20,
        tableHeaderFontSize: 9,
        abbreviatedFontSize: 8,
        skillRowHeight: 20,
        skillFontSize: 10,
        skillIconSize: 14,
        skillShowHeader: true,
        skillHeaderHeight: 18,
        skillHeaderFontSize: 8,
        skillAbbreviatedFontSize: 7,
        rowGlowMode: 'gradient-underline',
        skillRowGlowMode: 'gradient-underline',
        rowGlowOpacity: 0.5,
        skillRowGlowOpacity: 0.5,
        rowBorderRadius: 0,
        skillRowBorderRadius: 0,
      },
      header: {
        windowPadding: 6,
        headerPadding: 6,
        timerLabelFontSize: 10,
        timerFontSize: 14,
        sceneNameFontSize: 11,
        segmentFontSize: 10,
        resetButtonSize: 16,
        resetButtonPadding: 6,
        pauseButtonSize: 16,
        pauseButtonPadding: 6,
        bossOnlyButtonSize: 16,
        bossOnlyButtonPadding: 6,
        settingsButtonSize: 16,
        settingsButtonPadding: 6,
        minimizeButtonSize: 16,
        minimizeButtonPadding: 6,
        totalDamageLabelFontSize: 10,
        totalDamageValueFontSize: 14,
        totalDpsLabelFontSize: 10,
        totalDpsValueFontSize: 14,
        bossHealthLabelFontSize: 10,
        bossHealthNameFontSize: 11,
        bossHealthValueFontSize: 11,
        bossHealthPercentFontSize: 11,
        navTabFontSize: 9,
        navTabPaddingX: 8,
        navTabPaddingY: 4,
      }
    },
    medium: {
      name: "Medium",
      description: "Balanced size for most displays",
      table: {
        playerRowHeight: 28,
        playerFontSize: 13,
        playerIconSize: 20,
        showTableHeader: true,
        tableHeaderHeight: 24,
        tableHeaderFontSize: 11,
        abbreviatedFontSize: 10,
        skillRowHeight: 24,
        skillFontSize: 12,
        skillIconSize: 18,
        skillShowHeader: true,
        skillHeaderHeight: 22,
        skillHeaderFontSize: 10,
        skillAbbreviatedFontSize: 9,
        rowGlowMode: 'gradient-underline',
        skillRowGlowMode: 'gradient-underline',
        rowGlowOpacity: 0.5,
        skillRowGlowOpacity: 0.5,
        rowBorderRadius: 0,
        skillRowBorderRadius: 0,
      },
      header: {
        windowPadding: 12,
        headerPadding: 8,
        timerLabelFontSize: 12,
        timerFontSize: 18,
        sceneNameFontSize: 14,
        segmentFontSize: 12,
        resetButtonSize: 20,
        resetButtonPadding: 8,
        pauseButtonSize: 20,
        pauseButtonPadding: 8,
        bossOnlyButtonSize: 20,
        bossOnlyButtonPadding: 8,
        settingsButtonSize: 20,
        settingsButtonPadding: 8,
        minimizeButtonSize: 20,
        minimizeButtonPadding: 8,
        totalDamageLabelFontSize: 14,
        totalDamageValueFontSize: 18,
        totalDpsLabelFontSize: 14,
        totalDpsValueFontSize: 18,
        bossHealthLabelFontSize: 12,
        bossHealthNameFontSize: 14,
        bossHealthValueFontSize: 14,
        bossHealthPercentFontSize: 14,
        navTabFontSize: 11,
        navTabPaddingX: 12,
        navTabPaddingY: 6,
      }
    },
    large: {
      name: "Large",
      description: "Larger UI for high-res displays",
      table: {
        playerRowHeight: 36,
        playerFontSize: 16,
        playerIconSize: 26,
        showTableHeader: true,
        tableHeaderHeight: 30,
        tableHeaderFontSize: 13,
        abbreviatedFontSize: 12,
        skillRowHeight: 30,
        skillFontSize: 14,
        skillIconSize: 22,
        skillShowHeader: true,
        skillHeaderHeight: 26,
        skillHeaderFontSize: 12,
        skillAbbreviatedFontSize: 11,
        rowGlowMode: 'gradient-underline',
        skillRowGlowMode: 'gradient-underline',
        rowGlowOpacity: 0.5,
        skillRowGlowOpacity: 0.5,
        rowBorderRadius: 0,
        skillRowBorderRadius: 0,
      },
      header: {
        windowPadding: 16,
        headerPadding: 12,
        timerLabelFontSize: 14,
        timerFontSize: 24,
        sceneNameFontSize: 18,
        segmentFontSize: 14,
        resetButtonSize: 26,
        resetButtonPadding: 10,
        pauseButtonSize: 26,
        pauseButtonPadding: 10,
        bossOnlyButtonSize: 26,
        bossOnlyButtonPadding: 10,
        settingsButtonSize: 26,
        settingsButtonPadding: 10,
        minimizeButtonSize: 26,
        minimizeButtonPadding: 10,
        totalDamageLabelFontSize: 16,
        totalDamageValueFontSize: 24,
        totalDpsLabelFontSize: 16,
        totalDpsValueFontSize: 24,
        bossHealthLabelFontSize: 14,
        bossHealthNameFontSize: 18,
        bossHealthValueFontSize: 18,
        bossHealthPercentFontSize: 18,
        navTabFontSize: 13,
        navTabPaddingX: 16,
        navTabPaddingY: 8,
      }
    },
  };

  function applyColorPreset(presetKey: string) {
    const preset = COLOR_PRESETS[presetKey];
    if (preset) {
      // Save to settings (which will trigger the $effect to apply it)
      SETTINGS.accessibility.state.theme = preset.theme;
    }
  }

  function applySizePreset(presetKey: string) {
    const preset = SIZE_PRESETS[presetKey];
    if (preset) {
      // Apply table settings
      for (const [key, value] of Object.entries(preset.table)) {
        (SETTINGS.live.tableCustomization.state as any)[key] = value;
      }
      // Apply header settings
      for (const [key, value] of Object.entries(preset.header)) {
        (SETTINGS.live.headerCustomization.state as any)[key] = value;
      }
    }
  }

  let activeTab = $state('general');

  // Collapsible section state - all collapsed by default
  let expandedSections = $state({
    colorThemes: false,
    classSpecColors: false,
    backgroundImage: false,
    customFonts: false,
    liveDisplay: false,
    headerSettings: false,
    tableSettings: false,
    tableRowSettings: false,
    skillTableSettings: false,
  });

  function toggleSection(section: keyof typeof expandedSections) {
    expandedSections[section] = !expandedSections[section];
  }

  // Sync boss damage setting to backend
  let _mounted = false;
  onMount(() => {
    _mounted = true;
  });

  $effect(() => {
    if (_mounted) {
      void setBossOnlyDps(SETTINGS.live.general.state.bossOnlyDps);
    }
  });

  $effect(() => {
    if (_mounted) {
      void setDungeonSegmentsEnabled(SETTINGS.live.general.state.dungeonSegmentsEnabled);
    }
  });

  

  // Table size presets removed — sliders shown by default

  // Class/Spec colors tab state - 'class' or 'spec'
  let colorMode = $state<'class' | 'spec'>('class');

  // Sync useClassSpecColors setting with colorMode
  $effect(() => {
    SETTINGS.accessibility.state.useClassSpecColors = colorMode === 'spec';
  });

  // Group custom theme colors by category
  const colorCategories = $derived.by(() => {
    const categories: Record<string, string[]> = {};
    for (const [key, info] of Object.entries(CUSTOM_THEME_COLOR_LABELS)) {
      if (!categories[info.category]) {
        categories[info.category] = [];
      }
      categories[info.category]!.push(key);
    }
    return categories;
  });

  // Category order for display
  const categoryOrder = ['Base', 'Surfaces', 'Accents', 'Tables', 'Utility'];

  $effect(() => {
    setClickthrough(SETTINGS.accessibility.state.clickthrough);
  });

  function updateClassColor(className: string, color: string) {
    SETTINGS.accessibility.state.classColors = { ...SETTINGS.accessibility.state.classColors, [className]: color };
  }

  function updateClassSpecColor(specName: string, color: string) {
    SETTINGS.accessibility.state.classSpecColors = { ...SETTINGS.accessibility.state.classSpecColors, [specName]: color };
  }

  function resetClassColors() {
    SETTINGS.accessibility.state.classColors = { ...DEFAULT_CLASS_COLORS };
  }

  function resetClassSpecColors() {
    SETTINGS.accessibility.state.classSpecColors = { ...DEFAULT_CLASS_SPEC_COLORS };
  }

  

  function updateCustomThemeColor(key: string, value: string) {
    SETTINGS.accessibility.state.customThemeColors = { ...SETTINGS.accessibility.state.customThemeColors, [key]: value };
  }

  function resetCustomThemeColors() {
    SETTINGS.accessibility.state.customThemeColors = { ...DEFAULT_CUSTOM_THEME_COLORS };
  }

  // NOTE: preset theme selector removed — always show custom theme controls here
  // expose table customization state as any for optional skill-specific keys
  const tableCustomizationState: any = SETTINGS.live.tableCustomization.state;
</script>

<Tabs.Root bind:value={activeTab}>
  <Tabs.List>
    {#each themesTabs as themesTab (themesTab.id)}
      <Tabs.Trigger value={themesTab.id}>{themesTab.label}</Tabs.Trigger>
    {/each}
  </Tabs.List>

  {#if activeTab === 'general'}
    <Tabs.Content value="general">
      <div class="space-y-3">
        <!-- Color Themes Section -->
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('colorThemes')}
          >
            <h2 class="text-base font-semibold text-foreground">Color Themes</h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.colorThemes ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.colorThemes}
            <div class="px-4 pb-4 space-y-3">
              <div class="mt-3 pt-3 border-t border-border/50">
                  <div class="flex items-center justify-between mb-3">
                    <div>
                      <h3 class="text-sm font-semibold text-foreground">Custom Theme Colors</h3>
                      <p class="text-xs text-muted-foreground mt-0.5">Customize each color variable (with optional transparency)</p>
                    </div>
                    <button onclick={resetCustomThemeColors} class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors">Reset</button>
                  </div>
                  
                  {#each categoryOrder as category}
                    {#if colorCategories[category]}
                      <div class="mb-4">
                        <h4 class="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-2 px-1">{category}</h4>
                        <div class="space-y-1">
                          {#each colorCategories[category] ?? [] as colorKey}
                            {@const colorInfo = CUSTOM_THEME_COLOR_LABELS[colorKey]}
                            {#if colorInfo}
                              <SettingsColorAlpha
                                label={colorInfo.label}
                                description={colorInfo.description}
                                value={SETTINGS.accessibility.state.customThemeColors?.[colorKey] ?? DEFAULT_CUSTOM_THEME_COLORS[colorKey] ?? 'rgba(128, 128, 128, 1)'}
                                oninput={(value: string) => updateCustomThemeColor(colorKey, value)}
                              />
                            {/if}
                          {/each}
                        </div>
                      </div>
                    {/if}
                  {/each}
                </div>
            </div>
          {/if}
        </div>

        

        <!-- Class & Spec Colors Section -->
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('classSpecColors')}
          >
            <h2 class="text-base font-semibold text-foreground">Class & Spec Colors</h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.classSpecColors ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.classSpecColors}
            <div class="px-4 pb-4 space-y-3">
              <p class="text-xs text-muted-foreground">Customize colors for classes or specializations. Selecting "Spec Colors" enables spec-specific colors when spec is detected.</p>
              
              <!-- Tab buttons for Class/Spec -->
              <div class="flex items-center border border-border rounded-lg overflow-hidden bg-popover/30 w-fit">
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors {colorMode === 'class' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => colorMode = 'class'}
                >
                  Class Colors
                </button>
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors border-l border-border {colorMode === 'spec' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => colorMode = 'spec'}
                >
                  Spec Colors
                </button>
              </div>

              {#if colorMode === 'class'}
                <div class="flex items-center justify-between">
                  <p class="text-xs text-muted-foreground">Customize the color for each class displayed in the meter.</p>
                  <button onclick={resetClassColors} class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors">Reset</button>
                </div>
                <div class="grid grid-cols-2 gap-2 mt-2">
                  {#each CLASS_NAMES as className}
                    <label class="flex items-center gap-3 py-2 px-3 rounded-md hover:bg-popover/50 transition-colors">
                      <input type="color" value={getClassColorRaw(className)} oninput={(e) => updateClassColor(className, e.currentTarget.value)} class="w-8 h-8 rounded cursor-pointer border border-border/50" />
                      <span class="text-sm font-medium text-foreground">{className}</span>
                    </label>
                  {/each}
                </div>
              {:else}
                <div class="flex items-center justify-between">
                  <p class="text-xs text-muted-foreground">Customize colors for each specialization.</p>
                  <button onclick={resetClassSpecColors} class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors">Reset</button>
                </div>
                <div class="grid grid-cols-2 gap-2 mt-2">
                  {#each CLASS_SPEC_NAMES as specName}
                    <label class="flex items-center gap-3 py-2 px-3 rounded-md hover:bg-popover/50 transition-colors">
                      <input type="color" value={getClassColorRaw("", specName)} oninput={(e) => updateClassSpecColor(specName, e.currentTarget.value)} class="w-8 h-8 rounded cursor-pointer border border-border/50" />
                      <span class="text-sm font-medium text-foreground">{specName}</span>
                    </label>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Custom Fonts Section -->
        <!-- Table Row Settings (moved from Live > Table Settings) -->
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('tableRowSettings')}
          >
            <h2 class="text-base font-semibold text-foreground">Player Table Settings</h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.tableRowSettings ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.tableRowSettings}
            <div class="px-4 pb-4 space-y-3">
              <p class="text-xs text-muted-foreground">Controls for table row appearance and highlight mode. These settings apply to all live tables.</p>
              <div class="mt-2 space-y-2">
                <h4 class="text-sm font-medium text-foreground">Player Row</h4>
                <SettingsSlider
                  bind:value={SETTINGS.live.tableCustomization.state.playerRowHeight}
                  min={0} max={100} step={1}
                  label="Row Height"
                  description="Height of each player row in pixels"
                  unit="px"
                />
                <SettingsSlider
                  bind:value={SETTINGS.live.tableCustomization.state.playerFontSize}
                  min={0} max={100} step={1}
                  label="Font Size"
                  description="Font size for player names and stats"
                  unit="px"
                />
                <SettingsSlider
                  bind:value={SETTINGS.live.tableCustomization.state.playerIconSize}
                  min={0} max={100} step={1}
                  label="Icon Size"
                  description="Size of class/spec icons"
                  unit="px"
                />

                <div class="flex items-center gap-2">
                  <span class="text-sm text-muted-foreground">Mode</span>
                  <div class="flex items-center gap-1">
                    <button type="button" class="px-2 py-1 text-xs rounded {SETTINGS.live.tableCustomization.state.rowGlowMode === 'gradient-underline' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-popover/30'}" onclick={() => SETTINGS.live.tableCustomization.state.rowGlowMode = 'gradient-underline'}>Gradient w/ Underline</button>
                    <button type="button" class="px-2 py-1 text-xs rounded {SETTINGS.live.tableCustomization.state.rowGlowMode === 'gradient' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-popover/30'}" onclick={() => SETTINGS.live.tableCustomization.state.rowGlowMode = 'gradient'}>Gradient</button>
                    <button type="button" class="px-2 py-1 text-xs rounded {SETTINGS.live.tableCustomization.state.rowGlowMode === 'solid' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-popover/30'}" onclick={() => SETTINGS.live.tableCustomization.state.rowGlowMode = 'solid'}>Solid</button>
                  </div>
                </div>

                <SettingsSlider
                  bind:value={SETTINGS.live.tableCustomization.state.rowGlowOpacity}
                  min={0} max={1} step={0.01}
                  label="Row Glow Opacity"
                  description="Opacity applied to the row glow fill (0 = transparent, 1 = opaque)"
                />

                <SettingsSlider bind:value={SETTINGS.live.tableCustomization.state.rowBorderRadius} min={0} max={24} step={1} label="Row Border Radius" description="Rounded corner radius for row highlights" unit="px" />
              </div>
              <!-- Table Header & Number Styling -->
              <div class="space-y-4 pt-4 border-t border-border/30">
                

                <!-- Table Header Customization -->
                <div class="space-y-2 pt-3 border-t border-border/30">
                  <h3 class="text-sm font-semibold text-foreground">Table Header</h3>
                  <SettingsSwitch
                    bind:checked={SETTINGS.live.tableCustomization.state.showTableHeader}
                    label="Show Table Header"
                    description="Toggle visibility of the column headers"
                  />
                  {#if SETTINGS.live.tableCustomization.state.showTableHeader}
                    <SettingsSlider
                      bind:value={SETTINGS.live.tableCustomization.state.tableHeaderHeight}
                      min={0} max={100} step={1}
                      label="Header Height"
                      description="Height of the table header row"
                      unit="px"
                    />
                    <SettingsSlider
                      bind:value={SETTINGS.live.tableCustomization.state.tableHeaderFontSize}
                      min={0} max={100} step={1}
                      label="Header Font Size"
                      description="Font size for column header text"
                      unit="px"
                    />
                    <SettingsColor
                      bind:value={SETTINGS.live.tableCustomization.state.tableHeaderTextColor}
                      label="Header Text Color"
                      description="Color of column header text"
                    />
                  {/if}
                </div>

                <!-- Abbreviated Numbers -->
                <div class="space-y-2 pt-3 border-t border-border/30">
                  <h3 class="text-sm font-semibold text-foreground">Abbreviated Numbers (K, M, %)</h3>
                  <SettingsSlider
                    bind:value={SETTINGS.live.tableCustomization.state.abbreviatedFontSize}
                    min={0} max={100} step={1}
                    label="Suffix Font Size"
                    description="Font size for K, M, % symbols"
                    unit="px"
                  />
                </div>
              </div>
            </div>
          {/if}
        </div>
        <!-- Skill Table Settings -->
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('skillTableSettings')}
          >
            <h2 class="text-base font-semibold text-foreground">Skill Table Settings</h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.skillTableSettings ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.skillTableSettings}
            <div class="px-4 pb-4 space-y-4">
              <p class="text-xs text-muted-foreground">Customize skill table sizing, header and abbreviated number styling.</p>

              <div class="space-y-2 pt-3 border-t border-border/30">
                <h3 class="text-sm font-semibold text-foreground">Skill Row</h3>
                <SettingsSlider
                  bind:value={SETTINGS.live.tableCustomization.state.skillRowHeight}
                  min={0} max={100} step={1}
                  label="Skill Row Height"
                  description="Height of each skill row in pixels"
                  unit="px"
                />
                <SettingsSlider
                  bind:value={SETTINGS.live.tableCustomization.state.skillFontSize}
                  min={0} max={100} step={1}
                  label="Skill Font Size"
                  description="Font size for skill names and stats"
                  unit="px"
                />
                <SettingsSlider
                  bind:value={SETTINGS.live.tableCustomization.state.skillIconSize}
                  min={0} max={100} step={1}
                  label="Skill Icon Size"
                  description="Size of skill icons"
                  unit="px"
                />
                <div class="flex items-center gap-2 mt-2">
                  <span class="text-sm text-muted-foreground">Mode</span>
                  <div class="flex items-center gap-1">
                    <button type="button" class="px-2 py-1 text-xs rounded {tableCustomizationState.skillRowGlowMode === 'gradient-underline' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-popover/30'}" onclick={() => tableCustomizationState.skillRowGlowMode = 'gradient-underline'}>Gradient w/ Underline</button>
                    <button type="button" class="px-2 py-1 text-xs rounded {tableCustomizationState.skillRowGlowMode === 'gradient' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-popover/30'}" onclick={() => tableCustomizationState.skillRowGlowMode = 'gradient'}>Gradient</button>
                    <button type="button" class="px-2 py-1 text-xs rounded {tableCustomizationState.skillRowGlowMode === 'solid' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:bg-popover/30'}" onclick={() => tableCustomizationState.skillRowGlowMode = 'solid'}>Solid</button>
                  </div>
                </div>

                <SettingsSlider
                  bind:value={tableCustomizationState.skillRowGlowOpacity}
                  min={0} max={1} step={0.01}
                  label="Skill Row Glow Opacity"
                  description="Opacity applied to the skill row glow fill (0 = transparent, 1 = opaque)"
                />

                <SettingsSlider bind:value={tableCustomizationState.skillRowBorderRadius} min={0} max={24} step={1} label="Skill Row Border Radius" description="Rounded corner radius for skill row highlights" unit="px" />
              </div>

              <div class="space-y-2 pt-3 border-t border-border/30">
                <h3 class="text-sm font-semibold text-foreground">Skill Table Header</h3>
                <SettingsSwitch
                  bind:checked={SETTINGS.live.tableCustomization.state.skillShowHeader}
                  label="Show Skill Header"
                  description="Toggle visibility of skill table column headers"
                />
                {#if SETTINGS.live.tableCustomization.state.skillShowHeader}
                  <SettingsSlider
                    bind:value={SETTINGS.live.tableCustomization.state.skillHeaderHeight}
                    min={0} max={100} step={1}
                    label="Skill Header Height"
                    description="Height of the skill table header row"
                    unit="px"
                  />
                  <SettingsSlider
                    bind:value={SETTINGS.live.tableCustomization.state.skillHeaderFontSize}
                    min={0} max={100} step={1}
                    label="Skill Header Font Size"
                    description="Font size for skill column header text"
                    unit="px"
                  />
                  <SettingsColor
                    bind:value={SETTINGS.live.tableCustomization.state.skillHeaderTextColor}
                    label="Skill Header Text Color"
                    description="Color of skill column header text"
                  />
                {/if}
              </div>

              <div class="space-y-2 pt-3 border-t border-border/30">
                <h3 class="text-sm font-semibold text-foreground">Skill Abbreviated Numbers</h3>
                <SettingsSlider
                  bind:value={SETTINGS.live.tableCustomization.state.skillAbbreviatedFontSize}
                  min={0} max={100} step={1}
                  label="Skill Suffix Font Size"
                  description="Font size for K, M, % symbols in skill rows"
                  unit="px"
                />
              </div>
            </div>
          {/if}
        </div>
          <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
            <button
              type="button"
              class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
              onclick={() => toggleSection('backgroundImage')}
            >
              <h2 class="text-base font-semibold text-foreground">Background Image</h2>
              <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.backgroundImage ? 'rotate-180' : ''}" />
            </button>
            {#if expandedSections.backgroundImage}
              <div class="px-4 pb-4 space-y-2">
                <p class="text-xs text-muted-foreground">Use a custom image as the background for both main and live meter windows. NOTE: You must set your background as semi-transparent for this image to show.</p>
                <SettingsSwitch
                  bind:checked={SETTINGS.accessibility.state.backgroundImageEnabled}
                  label="Enable Background Image"
                  description="Use an image as the background"
                />
                {#if SETTINGS.accessibility.state.backgroundImageEnabled}
                  <div class="mt-2 space-y-2">
                    <SettingsFilePicker
                      label="Select Image"
                      description="Choose an image file (PNG, JPG, WebP)"
                      accept="image/*"
                      value={SETTINGS.accessibility.state.backgroundImage}
                      onchange={(dataUrl, _fileName) => {
                        SETTINGS.accessibility.state.backgroundImage = dataUrl;
                      }}
                      onclear={() => {
                        SETTINGS.accessibility.state.backgroundImage = '';
                      }}
                    />
                    <SettingsSelect
                      label="Image Mode"
                      description="How the image should fit the window"
                      bind:selected={SETTINGS.accessibility.state.backgroundImageMode}
                      values={["cover", "contain"]}
                    />
                    {#if SETTINGS.accessibility.state.backgroundImageMode === 'contain'}
                      <SettingsColorAlpha
                        label="Contain Fill Color"
                        description="Background color visible around the contained image"
                        value={SETTINGS.accessibility.state.backgroundImageContainColor}
                        oninput={(value: string) => {
                          SETTINGS.accessibility.state.backgroundImageContainColor = value;
                        }}
                      />
                    {/if}
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('customFonts')}
          >
            <h2 class="text-base font-semibold text-foreground">Custom Fonts</h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.customFonts ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.customFonts}
            <div class="px-4 pb-4 space-y-4">
              <p class="text-xs text-muted-foreground">Import custom fonts to replace the default fonts. Fonts should be .woff2, .woff, .ttf, or .otf files.</p>
              
              <!-- Sans-serif Font -->
              <div class="space-y-2 pt-2 border-t border-border/30">
                <h3 class="text-sm font-semibold text-foreground">Sans-serif Font (UI Text)</h3>
                <p class="text-xs text-muted-foreground">Default: Inter Variable</p>
                <SettingsSwitch
                  bind:checked={SETTINGS.accessibility.state.customFontSansEnabled}
                  label="Enable Custom Sans Font"
                  description="Use a custom font for UI text"
                />
                {#if SETTINGS.accessibility.state.customFontSansEnabled}
                  <SettingsFilePicker
                    label="Select Font File"
                    description="Choose a font file (.woff2, .woff, .ttf, .otf)"
                    accept=".woff2,.woff,.ttf,.otf"
                    value={SETTINGS.accessibility.state.customFontSansUrl}
                    onchange={(dataUrl, fileName) => {
                      SETTINGS.accessibility.state.customFontSansUrl = dataUrl;
                      // Extract font name from file name (remove extension)
                      const fontName = fileName.replace(/\.(woff2?|ttf|otf)$/i, '');
                      SETTINGS.accessibility.state.customFontSansName = fontName;
                      // Register the font face
                      const fontFace = new FontFace(fontName, `url(${dataUrl})`);
                      fontFace.load().then((loadedFace) => {
                        document.fonts.add(loadedFace);
                      }).catch(e => console.error('Failed to load font:', e));
                    }}
                    onclear={() => {
                      SETTINGS.accessibility.state.customFontSansUrl = '';
                      SETTINGS.accessibility.state.customFontSansName = '';
                    }}
                  />
                  {#if SETTINGS.accessibility.state.customFontSansName}
                    <p class="text-xs text-muted-foreground pl-3">Loaded: {SETTINGS.accessibility.state.customFontSansName}</p>
                  {/if}
                {/if}
              </div>

              <!-- Monospace Font -->
              <div class="space-y-2 pt-3 border-t border-border/30">
                <h3 class="text-sm font-semibold text-foreground">Monospace Font (Numbers, Code)</h3>
                <p class="text-xs text-muted-foreground">Default: Geist Mono Variable</p>
                <SettingsSwitch
                  bind:checked={SETTINGS.accessibility.state.customFontMonoEnabled}
                  label="Enable Custom Mono Font"
                  description="Use a custom font for numbers and code"
                />
                {#if SETTINGS.accessibility.state.customFontMonoEnabled}
                  <SettingsFilePicker
                    label="Select Font File"
                    description="Choose a font file (.woff2, .woff, .ttf, .otf)"
                    accept=".woff2,.woff,.ttf,.otf"
                    value={SETTINGS.accessibility.state.customFontMonoUrl}
                    onchange={(dataUrl, fileName) => {
                      SETTINGS.accessibility.state.customFontMonoUrl = dataUrl;
                      // Extract font name from file name (remove extension)
                      const fontName = fileName.replace(/\.(woff2?|ttf|otf)$/i, '');
                      SETTINGS.accessibility.state.customFontMonoName = fontName;
                      // Register the font face
                      const fontFace = new FontFace(fontName, `url(${dataUrl})`);
                      fontFace.load().then((loadedFace) => {
                        document.fonts.add(loadedFace);
                      }).catch(e => console.error('Failed to load font:', e));
                    }}
                    onclear={() => {
                      SETTINGS.accessibility.state.customFontMonoUrl = '';
                      SETTINGS.accessibility.state.customFontMonoName = '';
                    }}
                  />
                  {#if SETTINGS.accessibility.state.customFontMonoName}
                    <p class="text-xs text-muted-foreground pl-3">Loaded: {SETTINGS.accessibility.state.customFontMonoName}</p>
                  {/if}
                {/if}
              </div>
            </div>
          {/if}
        </div>
      </div>
    </Tabs.Content>
  
  {:else if activeTab === 'live'}
    <Tabs.Content value="live">
      <div class="space-y-3">

        <!-- Live Meter Display Settings -->
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('liveDisplay')}
          >
            <h2 class="text-base font-semibold text-foreground">Live Meter Display Settings</h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.liveDisplay ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.liveDisplay}
            <div class="px-4 pb-4 space-y-2">
              <SettingsSwitch
                bind:checked={SETTINGS.accessibility.state.clickthrough}
                label="Clickthrough Mode"
                description={SETTINGS.accessibility.state.clickthrough ? 'Clickthrough Enabled - Mouse clicks pass through window' : 'Enable Clickthrough Mode'}
              />
            </div>
          {/if}
        </div>

        <!-- Header Settings -->
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('headerSettings')}
          >
            <h2 class="text-base font-semibold text-foreground">Header Settings</h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.headerSettings ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.headerSettings}
            <div class="px-4 pb-4 space-y-4">
              <!-- Custom Header Settings -->
              <div class="space-y-4 pt-2 border-t border-border/50">
                  <!-- Layout & Padding -->
                  <div class="space-y-2">
                    <div class="flex items-center justify-between">
                      <h3 class="text-sm font-semibold text-foreground">Layout & Padding</h3>
                    </div>
                    <SettingsSlider
                      bind:value={SETTINGS.live.headerCustomization.state.windowPadding}
                      min={0} max={24} step={1}
                      label="Window Padding"
                      description="Padding around the entire live meter window"
                      unit="px"
                    />
                    <SettingsSlider
                      bind:value={SETTINGS.live.headerCustomization.state.headerPadding}
                      min={0} max={16} step={1}
                      label="Header Internal Padding"
                      description="Padding within the header area"
                      unit="px"
                    />
                  </div>

                  <!-- Timer Settings -->
                  <div class="space-y-2 pt-3 border-t border-border/30">
                    <h3 class="text-sm font-semibold text-foreground">Timer</h3>
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showTimer}
                      label="Show Timer"
                      description="Display the encounter timer"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showTimer}
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.timerLabelFontSize}
                        min={0} max={20} step={1}
                        label="Label Font Size"
                        description="Font size for 'Timer' label (0 to hide)"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.timerFontSize}
                        min={10} max={32} step={1}
                        label="Timer Font Size"
                        description="Font size for the timer value"
                        unit="px"
                      />
                    {/if}
                  </div>

                  <!-- Scene Name -->
                  <div class="space-y-2 pt-3 border-t border-border/30">
                    <h3 class="text-sm font-semibold text-foreground">Scene Name</h3>
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showSceneName}
                      label="Show Scene Name"
                      description="Display the current dungeon/scene name"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showSceneName}
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.sceneNameFontSize}
                        min={10} max={24} step={1}
                        label="Scene Name Font Size"
                        description="Font size for scene name"
                        unit="px"
                      />
                    {/if}
                  </div>

                  <!-- Segment Info -->
                  <div class="space-y-2 pt-3 border-t border-border/30">
                    <h3 class="text-sm font-semibold text-foreground">Segment Info</h3>
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showSegmentInfo}
                      label="Show Segment Info"
                      description="Display boss/trash segment indicator"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showSegmentInfo}
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.segmentFontSize}
                        min={8} max={18} step={1}
                        label="Segment Font Size"
                        description="Font size for segment badge text"
                        unit="px"
                      />
                    {/if}
                  </div>

                  <!-- Control Buttons -->
                  <div class="space-y-2 pt-3 border-t border-border/30">
                    <h3 class="text-sm font-semibold text-foreground">Control Buttons</h3>
                    
                    <!-- Reset Button -->
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showResetButton}
                      label="Show Reset Button"
                      description="Button to reset the encounter"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showResetButton}
                      <div class="grid grid-cols-2 gap-2 pl-4">
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.resetButtonSize}
                          min={12} max={32} step={1}
                          label="Icon Size"
                          unit="px"
                        />
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.resetButtonPadding}
                          min={2} max={16} step={1}
                          label="Padding"
                          unit="px"
                        />
                      </div>
                    {/if}
                    
                    <!-- Pause Button -->
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showPauseButton}
                      label="Show Pause Button"
                      description="Button to pause/resume the encounter"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showPauseButton}
                      <div class="grid grid-cols-2 gap-2 pl-4">
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.pauseButtonSize}
                          min={12} max={32} step={1}
                          label="Icon Size"
                          unit="px"
                        />
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.pauseButtonPadding}
                          min={2} max={16} step={1}
                          label="Padding"
                          unit="px"
                        />
                      </div>
                    {/if}
                    
                    <!-- Boss Only Button -->
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showBossOnlyButton}
                      label="Show Boss Only Button"
                      description="Button to toggle boss-only damage mode"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showBossOnlyButton}
                      <div class="grid grid-cols-2 gap-2 pl-4">
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.bossOnlyButtonSize}
                          min={12} max={32} step={1}
                          label="Icon Size"
                          unit="px"
                        />
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.bossOnlyButtonPadding}
                          min={2} max={16} step={1}
                          label="Padding"
                          unit="px"
                        />
                      </div>
                    {/if}
                    
                    <!-- Settings Button -->
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showSettingsButton}
                      label="Show Settings Button"
                      description="Button to open settings window"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showSettingsButton}
                      <div class="grid grid-cols-2 gap-2 pl-4">
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.settingsButtonSize}
                          min={12} max={32} step={1}
                          label="Icon Size"
                          unit="px"
                        />
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.settingsButtonPadding}
                          min={2} max={16} step={1}
                          label="Padding"
                          unit="px"
                        />
                      </div>
                    {/if}
                    
                    <!-- Minimize Button -->
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showMinimizeButton}
                      label="Show Minimize Button"
                      description="Button to minimize the live meter"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showMinimizeButton}
                      <div class="grid grid-cols-2 gap-2 pl-4">
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.minimizeButtonSize}
                          min={12} max={32} step={1}
                          label="Icon Size"
                          unit="px"
                        />
                        <SettingsSlider
                          bind:value={SETTINGS.live.headerCustomization.state.minimizeButtonPadding}
                          min={2} max={16} step={1}
                          label="Padding"
                          unit="px"
                        />
                      </div>
                    {/if}
                  </div>

                  <!-- Total Damage -->
                  <div class="space-y-2 pt-3 border-t border-border/30">
                    <h3 class="text-sm font-semibold text-foreground">Total Damage</h3>
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showTotalDamage}
                      label="Show Total Damage"
                      description="Display total damage dealt"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showTotalDamage}
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.totalDamageLabelFontSize}
                        min={8} max={20} step={1}
                        label="Label Font Size"
                        description="Font size for 'T.DMG' label"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.totalDamageValueFontSize}
                        min={10} max={32} step={1}
                        label="Value Font Size"
                        description="Font size for damage value"
                        unit="px"
                      />
                    {/if}
                  </div>

                  <!-- Total DPS -->
                  <div class="space-y-2 pt-3 border-t border-border/30">
                    <h3 class="text-sm font-semibold text-foreground">Total DPS</h3>
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showTotalDps}
                      label="Show Total DPS"
                      description="Display total damage per second"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showTotalDps}
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.totalDpsLabelFontSize}
                        min={8} max={20} step={1}
                        label="Label Font Size"
                        description="Font size for 'T.DPS' label"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.totalDpsValueFontSize}
                        min={10} max={32} step={1}
                        label="Value Font Size"
                        description="Font size for DPS value"
                        unit="px"
                      />
                    {/if}
                  </div>

                  <!-- Boss Health -->
                  <div class="space-y-2 pt-3 border-t border-border/30">
                    <h3 class="text-sm font-semibold text-foreground">Boss Health</h3>
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showBossHealth}
                      label="Show Boss Health"
                      description="Display current boss health bar"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showBossHealth}
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.bossHealthLabelFontSize}
                        min={0} max={20} step={1}
                        label="Label Font Size"
                        description="Font size for 'BOSS' label"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.bossHealthNameFontSize}
                        min={0} max={24} step={1}
                        label="Boss Name Font Size"
                        description="Font size for boss name"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.bossHealthValueFontSize}
                        min={0} max={24} step={1}
                        label="HP Value Font Size"
                        description="Font size for HP values (1.5M / 3M)"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.bossHealthPercentFontSize}
                        min={0} max={24} step={1}
                        label="Percentage Font Size"
                        description="Font size for HP percentage"
                        unit="px"
                      />
                    {/if}
                  </div>

                  <!-- Navigation Tabs -->
                  <div class="space-y-2 pt-3 border-t border-border/30">
                    <h3 class="text-sm font-semibold text-foreground">Navigation Tabs (DPS/HEAL/TANKED)</h3>
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showNavigationTabs}
                      label="Show Navigation Tabs"
                      description="Display DPS/HEAL/TANKED tab buttons"
                    />
                    {#if SETTINGS.live.headerCustomization.state.showNavigationTabs}
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.navTabFontSize}
                        min={8} max={18} step={1}
                        label="Tab Font Size"
                        description="Font size for tab text"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.navTabPaddingX}
                        min={4} max={24} step={1}
                        label="Horizontal Padding"
                        description="Left/right padding inside tabs"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.navTabPaddingY}
                        min={2} max={16} step={1}
                        label="Vertical Padding"
                        description="Top/bottom padding inside tabs"
                        unit="px"
                      />
                    {/if}
                  </div>
                </div>
            </div>
          {/if}
        </div>

        
      </div>
    </Tabs.Content>

  {:else if activeTab === 'presets'}
    <Tabs.Content value="presets">
      <div class="space-y-6">
        <!-- Color Theme Presets -->
        <div class="space-y-3">
          <div>
            <h2 class="text-base font-semibold text-foreground">Color Themes</h2>
            <p class="text-xs text-muted-foreground mt-0.5">Choose a color preset to apply instantly</p>
          </div>
          <div class="grid grid-cols-2 gap-3">
            {#each Object.entries(COLOR_PRESETS) as [key, preset]}
              <button
                type="button"
                class="group relative flex flex-col items-start p-4 rounded-lg border border-border/60 bg-card/40 hover:bg-card/60 hover:border-primary/50 transition-all text-left"
                onclick={() => applyColorPreset(key)}
              >
                <!-- Color preview dots -->
                <div class="flex gap-1.5 mb-2">
                  <span class="w-4 h-4 rounded-full border border-black/20" style="background: {preset.preview.bg}"></span>
                  <span class="w-4 h-4 rounded-full border border-black/20" style="background: {preset.preview.primary}"></span>
                  <span class="w-4 h-4 rounded-full border border-black/20" style="background: {preset.preview.accent}"></span>
                  <span class="w-4 h-4 rounded-full border border-black/20" style="background: {preset.preview.fg}"></span>
                </div>
                <span class="text-sm font-medium text-foreground">{preset.name}</span>
                <span class="text-xs text-muted-foreground mt-0.5">{preset.description}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Size Presets -->
        <div class="space-y-3 pt-4 border-t border-border/50">
          <div>
            <h2 class="text-base font-semibold text-foreground">Size Presets</h2>
            <p class="text-xs text-muted-foreground mt-0.5">Adjust table and header sizing for your display</p>
          </div>
          <div class="grid grid-cols-4 gap-3">
            {#each Object.entries(SIZE_PRESETS) as [key, preset]}
              <button
                type="button"
                class="group flex flex-col items-center justify-center p-4 rounded-lg border border-border/60 bg-card/40 hover:bg-card/60 hover:border-primary/50 transition-all"
                onclick={() => applySizePreset(key)}
              >
                <!-- Size indicator -->
                <div class="flex items-end gap-0.5 mb-2 h-6">
                  {#if key === 'compact'}
                    <span class="w-2 h-2 bg-primary/60 rounded-sm"></span>
                    <span class="w-2 h-3 bg-primary/40 rounded-sm"></span>
                    <span class="w-2 h-4 bg-primary/20 rounded-sm"></span>
                    <span class="w-2 h-5 bg-primary/10 rounded-sm"></span>
                  {:else if key === 'small'}
                    <span class="w-2 h-2 bg-primary/40 rounded-sm"></span>
                    <span class="w-2 h-3 bg-primary/60 rounded-sm"></span>
                    <span class="w-2 h-4 bg-primary/30 rounded-sm"></span>
                    <span class="w-2 h-5 bg-primary/10 rounded-sm"></span>
                  {:else if key === 'medium'}
                    <span class="w-2 h-2 bg-primary/20 rounded-sm"></span>
                    <span class="w-2 h-3 bg-primary/40 rounded-sm"></span>
                    <span class="w-2 h-4 bg-primary/60 rounded-sm"></span>
                    <span class="w-2 h-5 bg-primary/30 rounded-sm"></span>
                  {:else}
                    <span class="w-2 h-2 bg-primary/10 rounded-sm"></span>
                    <span class="w-2 h-3 bg-primary/20 rounded-sm"></span>
                    <span class="w-2 h-4 bg-primary/40 rounded-sm"></span>
                    <span class="w-2 h-5 bg-primary/60 rounded-sm"></span>
                  {/if}
                </div>
                <span class="text-sm font-medium text-foreground">{preset.name}</span>
                <span class="text-xs text-muted-foreground mt-0.5 text-center">{preset.description}</span>
              </button>
            {/each}
          </div>
        </div>
      </div>
    </Tabs.Content>
  {/if}
</Tabs.Root>