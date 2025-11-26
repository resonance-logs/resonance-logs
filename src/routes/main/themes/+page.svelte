<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSelect from "../settings/settings-select.svelte";
  import SettingsSlider from "../settings/settings-slider.svelte";
  import SettingsSwitch from "../settings/settings-switch.svelte";
  import SettingsColor from "../settings/settings-color.svelte";
  import SettingsColorAlpha from "../settings/settings-color-alpha.svelte";
  import { SETTINGS, AVAILABLE_THEMES, DEFAULT_CLASS_COLORS, DEFAULT_CLASS_SPEC_COLORS, CLASS_SPEC_NAMES, DEFAULT_LIVE_TABLE_SETTINGS, DEFAULT_CUSTOM_THEME_COLORS, CUSTOM_THEME_COLOR_LABELS, DEFAULT_HEADER_SETTINGS, HEADER_PRESETS } from "$lib/settings-store";
  import { setClickthrough, CLASS_NAMES, getClassColorRaw } from "$lib/utils.svelte";
  import ChevronDown from "virtual:icons/lucide/chevron-down";

  const themesTabs = [
    { id: "general", label: "General" },
    { id: "main", label: "Main" },
    { id: "live", label: "Live" },
  ];

  let activeTab = $state('general');

  // Collapsible section state - all collapsed by default
  let expandedSections = $state({
    colorThemes: false,
    classSpecColors: false,
    transparency: false,
    liveDisplay: false,
    headerSettings: false,
    tableSettings: false,
  });

  function toggleSection(section: keyof typeof expandedSections) {
    expandedSections[section] = !expandedSections[section];
  }

  // Header presets
  type HeaderPreset = 'full' | 'compact' | 'none' | 'custom';
  let headerPreset = $state<HeaderPreset>('full');

  function applyHeaderPreset(preset: 'full' | 'compact' | 'none') {
    const settings = HEADER_PRESETS[preset];
    Object.assign(SETTINGS.live.headerCustomization.state, settings);
  }

  function handleHeaderPresetChange(preset: HeaderPreset) {
    headerPreset = preset;
    if (preset !== 'custom') {
      applyHeaderPreset(preset);
    }
  }

  function resetHeaderSettings() {
    Object.assign(SETTINGS.live.headerCustomization.state, DEFAULT_HEADER_SETTINGS);
  }

  // Table size presets
  type TableSizePreset = 'large' | 'medium' | 'small' | 'custom';
  let tableSizePreset = $state<TableSizePreset>('medium');

  const TABLE_PRESETS = {
    large: {
      playerRowHeight: 36,
      playerFontSize: 15,
      playerIconSize: 24,
      showTableHeader: true,
      tableHeaderHeight: 28,
      tableHeaderFontSize: 13,
      abbreviatedFontSize: 12,
      skillRowHeight: 32,
      skillFontSize: 14,
      skillIconSize: 22,
      skillShowHeader: true,
      skillHeaderHeight: 26,
      skillHeaderFontSize: 12,
      skillAbbreviatedFontSize: 11,
    },
    medium: {
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
    },
    small: {
      playerRowHeight: 22,
      playerFontSize: 11,
      playerIconSize: 16,
      showTableHeader: true,
      tableHeaderHeight: 20,
      tableHeaderFontSize: 9,
      abbreviatedFontSize: 8,
      skillRowHeight: 18,
      skillFontSize: 10,
      skillIconSize: 14,
      skillShowHeader: true,
      skillHeaderHeight: 18,
      skillHeaderFontSize: 8,
      skillAbbreviatedFontSize: 7,
    },
  };

  function applyTablePreset(preset: 'large' | 'medium' | 'small') {
    const settings = TABLE_PRESETS[preset];
    Object.assign(SETTINGS.live.tableCustomization.state, settings);
  }

  function handlePresetChange(preset: TableSizePreset) {
    tableSizePreset = preset;
    if (preset !== 'custom') {
      applyTablePreset(preset);
    }
  }

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
  const categoryOrder = ['Base', 'Surfaces', 'Accents', 'Utility', 'Charts', 'Sidebar'];

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

  function resetTableCustomization() {
    Object.assign(SETTINGS.live.tableCustomization.state, DEFAULT_LIVE_TABLE_SETTINGS);
  }

  function updateCustomThemeColor(key: string, value: string) {
    SETTINGS.accessibility.state.customThemeColors = { ...SETTINGS.accessibility.state.customThemeColors, [key]: value };
  }

  function resetCustomThemeColors() {
    SETTINGS.accessibility.state.customThemeColors = { ...DEFAULT_CUSTOM_THEME_COLORS };
  }

  // Check if custom theme is selected
  let isCustomTheme = $derived(SETTINGS.accessibility.state.theme === 'custom');
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
              <SettingsSelect
                label="Theme"
                description="Choose one of the built-in themes"
                bind:selected={SETTINGS.accessibility.state["theme"]}
                values={AVAILABLE_THEMES}
              />

              {#if isCustomTheme}
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
              {/if}
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
      </div>
    </Tabs.Content>
  {:else if activeTab === 'main'}
    <Tabs.Content value="main">
      <div class="space-y-3">
        <!-- Main tab content will go here -->
      </div>
    </Tabs.Content>
  {:else if activeTab === 'live'}
    <Tabs.Content value="live">
      <div class="space-y-3">
        <!-- Transparency Settings -->
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('transparency')}
          >
            <h2 class="text-base font-semibold text-foreground">Transparency Settings</h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.transparency ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.transparency}
            <div class="px-4 pb-4 space-y-2">
              <p class="text-xs text-muted-foreground">Make the live meter window transparent.</p>
              <SettingsSwitch
                bind:checked={SETTINGS.accessibility.state.transparency}
                label="Transparent Mode"
                description={SETTINGS.accessibility.state.transparency ? 'Transparent Mode Enabled' : 'Enable Transparent Mode'}
              />
              <SettingsSlider bind:value={SETTINGS.accessibility.state["transparentOpacityPercent"]} min={0} max={100} step={1} label="Transparency Opacity" description="Lower values make the meter more see-through. 0% is fully transparent." unit="%" />
            </div>
          {/if}
        </div>

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
              <SettingsSwitch
                bind:checked={SETTINGS.live.general.state.useDummyData}
                label="Use Dummy Data"
                description="Inject dummy player data into the live meter for testing and preview purposes"
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
              <p class="text-xs text-muted-foreground">Choose a preset or customize individual header elements.</p>
              
              <!-- Header Preset Selector -->
              <div class="flex items-center border border-border rounded-lg overflow-hidden bg-popover/30 w-fit">
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors {headerPreset === 'full' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => handleHeaderPresetChange('full')}
                >
                  Full
                </button>
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors border-l border-border {headerPreset === 'compact' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => handleHeaderPresetChange('compact')}
                >
                  Compact
                </button>
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors border-l border-border {headerPreset === 'none' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => handleHeaderPresetChange('none')}
                >
                  None
                </button>
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors border-l border-border {headerPreset === 'custom' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => handleHeaderPresetChange('custom')}
                >
                  Custom
                </button>
              </div>

              {#if headerPreset === 'custom'}
                <!-- Custom Header Settings -->
                <div class="space-y-4 pt-2 border-t border-border/50">
                  <!-- Layout & Padding -->
                  <div class="space-y-2">
                    <div class="flex items-center justify-between">
                      <h3 class="text-sm font-semibold text-foreground">Layout & Padding</h3>
                      <button onclick={resetHeaderSettings} class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors">Reset All</button>
                    </div>
                    <SettingsSwitch
                      bind:checked={SETTINGS.live.headerCustomization.state.showHeader}
                      label="Show Header"
                      description="Toggle visibility of the entire header"
                    />
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
                        min={8} max={20} step={1}
                        label="Label Font Size"
                        description="Font size for 'BOSS' label"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.bossHealthNameFontSize}
                        min={10} max={24} step={1}
                        label="Boss Name Font Size"
                        description="Font size for boss name"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.bossHealthValueFontSize}
                        min={10} max={24} step={1}
                        label="HP Value Font Size"
                        description="Font size for HP values (1.5M / 3M)"
                        unit="px"
                      />
                      <SettingsSlider
                        bind:value={SETTINGS.live.headerCustomization.state.bossHealthPercentFontSize}
                        min={10} max={24} step={1}
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
              {/if}
            </div>
          {/if}
        </div>

        <!-- Table Settings -->
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('tableSettings')}
          >
            <h2 class="text-base font-semibold text-foreground">Table Settings</h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.tableSettings ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.tableSettings}
            <div class="px-4 pb-4 space-y-4">
              <p class="text-xs text-muted-foreground">Choose a preset size or customize individual settings.</p>
              
              <!-- Size Preset Selector -->
              <div class="flex items-center border border-border rounded-lg overflow-hidden bg-popover/30 w-fit">
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors {tableSizePreset === 'large' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => handlePresetChange('large')}
                >
                  Large
                </button>
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors border-l border-border {tableSizePreset === 'medium' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => handlePresetChange('medium')}
                >
                  Medium
                </button>
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors border-l border-border {tableSizePreset === 'small' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => handlePresetChange('small')}
                >
                  Small
                </button>
                <button
                  type="button"
                  class="px-4 py-2 text-sm font-medium transition-colors border-l border-border {tableSizePreset === 'custom' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-popover/60'}"
                  onclick={() => handlePresetChange('custom')}
                >
                  Custom
                </button>
              </div>

              {#if tableSizePreset === 'custom'}
                <!-- Custom Settings -->
                <div class="space-y-4 pt-2 border-t border-border/50">
                  <!-- Player Row Customization -->
                  <div class="space-y-2">
                    <div class="flex items-center justify-between">
                      <h3 class="text-sm font-semibold text-foreground">Player Row</h3>
                      <button onclick={resetTableCustomization} class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors">Reset All</button>
                    </div>
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
                    <SettingsColor
                      bind:value={SETTINGS.live.tableCustomization.state.playerTextColor}
                      label="Text Color"
                      description="Color of player names and stat values"
                    />
                  </div>

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
                    <SettingsColor
                      bind:value={SETTINGS.live.tableCustomization.state.abbreviatedColor}
                      label="Suffix Color"
                      description="Color of K, M, % symbols"
                    />
                  </div>

                  <!-- Skill Row Customization -->
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
                    <SettingsColor
                      bind:value={SETTINGS.live.tableCustomization.state.skillTextColor}
                      label="Skill Text Color"
                      description="Color of skill names and stat values"
                    />
                  </div>

                  <!-- Skill Header Customization -->
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

                  <!-- Skill Abbreviated Numbers -->
                  <div class="space-y-2 pt-3 border-t border-border/30">
                    <h3 class="text-sm font-semibold text-foreground">Skill Abbreviated Numbers</h3>
                    <SettingsSlider
                      bind:value={SETTINGS.live.tableCustomization.state.skillAbbreviatedFontSize}
                      min={0} max={100} step={1}
                      label="Skill Suffix Font Size"
                      description="Font size for K, M, % symbols in skill rows"
                      unit="px"
                    />
                    <SettingsColor
                      bind:value={SETTINGS.live.tableCustomization.state.skillAbbreviatedColor}
                      label="Skill Suffix Color"
                      description="Color of K, M, % symbols in skill rows"
                    />
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </Tabs.Content>
  {/if}
</Tabs.Root>