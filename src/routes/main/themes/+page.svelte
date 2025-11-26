<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSelect from "../settings/settings-select.svelte";
  import SettingsSlider from "../settings/settings-slider.svelte";
  import SettingsSwitch from "../settings/settings-switch.svelte";
  import SettingsColor from "../settings/settings-color.svelte";
  import { SETTINGS, AVAILABLE_THEMES, DEFAULT_CLASS_COLORS, DEFAULT_CLASS_SPEC_COLORS, CLASS_SPEC_NAMES, DEFAULT_LIVE_TABLE_SETTINGS } from "$lib/settings-store";
  import { setClickthrough, CLASS_NAMES, getClassColorRaw } from "$lib/utils.svelte";

  const themesTabs = [
    { id: "general", label: "General" },
    { id: "main", label: "Main" },
    { id: "live", label: "Live" },
  ];

  let activeTab = $state('general');

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
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-semibold text-foreground">Class Colors</h2>
              <p class="text-xs text-muted-foreground mt-1">Customize the color for each class displayed in the meter.</p>
            </div>
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
        </div>

        <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-semibold text-foreground">Spec Colors</h2>
              <p class="text-xs text-muted-foreground mt-1">Use spec-specific colors instead of class colors when spec is detected.</p>
            </div>
            <button onclick={resetClassSpecColors} class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors">Reset</button>
          </div>
          <SettingsSwitch bind:checked={SETTINGS.accessibility.state.useClassSpecColors} label="Use Spec Colors" description={SETTINGS.accessibility.state.useClassSpecColors ? 'Spec colors enabled' : 'Enable spec-specific colors'} />
          {#if SETTINGS.accessibility.state.useClassSpecColors}
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

        <!-- Player Row Customization -->
        <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-semibold text-foreground">Player Row Customization</h2>
              <p class="text-xs text-muted-foreground mt-1">Customize the appearance of player rows in the live meter table.</p>
            </div>
            <button onclick={resetTableCustomization} class="px-3 py-1.5 text-xs font-medium rounded-md bg-muted hover:bg-muted/80 text-muted-foreground transition-colors">Reset All</button>
          </div>
          <div class="space-y-2">
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
        </div>

        <!-- Table Header Customization -->
        <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
          <div>
            <h2 class="text-base font-semibold text-foreground">Table Header Customization</h2>
            <p class="text-xs text-muted-foreground mt-1">Customize the table header appearance.</p>
          </div>
          <div class="space-y-2">
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
        </div>

        <!-- Abbreviated Numbers Customization -->
        <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
          <div>
            <h2 class="text-base font-semibold text-foreground">Abbreviated Numbers (K, M, %)</h2>
            <p class="text-xs text-muted-foreground mt-1">Customize the appearance of abbreviated number suffixes.</p>
          </div>
          <div class="space-y-2">
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
        </div>

        <!-- Skill Row Customization -->
        <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
          <div>
            <h2 class="text-base font-semibold text-foreground">Skill Row Customization</h2>
            <p class="text-xs text-muted-foreground mt-1">Customize the appearance of skill breakdown rows (separate from player rows).</p>
          </div>
          <div class="space-y-2">
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
        </div>

        <!-- Skill Header Customization -->
        <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
          <div>
            <h2 class="text-base font-semibold text-foreground">Skill Table Header Customization</h2>
            <p class="text-xs text-muted-foreground mt-1">Customize the skill breakdown table header appearance.</p>
          </div>
          <div class="space-y-2">
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
        </div>

        <!-- Skill Abbreviated Numbers -->
        <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
          <div>
            <h2 class="text-base font-semibold text-foreground">Skill Abbreviated Numbers</h2>
            <p class="text-xs text-muted-foreground mt-1">Customize abbreviated number suffixes for skill rows.</p>
          </div>
          <div class="space-y-2">
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
      </div>
    </Tabs.Content>
  {/if}
</Tabs.Root>
