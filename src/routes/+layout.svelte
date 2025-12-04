<script lang="ts">
  /**
   * @file This is the root layout for the application.
   * It imports the global stylesheet and disables the context menu.
   */
  import "../app.css";
  import { SETTINGS } from "$lib/settings-store";
  import { setBossOnlyDps, setDungeonSegmentsEnabled } from "$lib/api";
  // Only allow warnings and errors to be printed to console in production builds
  if (typeof window !== 'undefined' && import.meta.env.PROD) {
    // Keep warn and error; disable verbose logging
    console.log = (..._args: any[]) => {};
    console.debug = (..._args: any[]) => {};
    console.info = (..._args: any[]) => {};
  }

  let { children } = $props();
  let lastBossOnlySync: boolean | null = null;
  let lastDungeonSegmentsSync: boolean | null = null;

  const customThemeKeyToCssVar: Record<string, string> = {
    backgroundMain: '--background-main',
    backgroundLive: '--background-live',
    foreground: '--foreground',
    card: '--card',
    cardForeground: '--card-foreground',
    popover: '--popover',
    popoverForeground: '--popover-foreground',
    primary: '--primary',
    primaryForeground: '--primary-foreground',
    secondary: '--secondary',
    secondaryForeground: '--secondary-foreground',
    muted: '--muted',
    mutedForeground: '--muted-foreground',
    accent: '--accent',
    accentForeground: '--accent-foreground',
    destructive: '--destructive',
    destructiveForeground: '--destructive-foreground',
    border: '--border',
    input: '--input',
    ring: '--ring',
    tooltipBg: '--tooltip-bg',
    tooltipBorder: '--tooltip-border',
    tooltipFg: '--tooltip-fg',
    playerTextColor: '--player-text-color',
    skillTextColor: '--skill-text-color',
    abbreviatedColor: '--abbreviated-color',
    skillAbbreviatedColor: '--skill-abbreviated-color',
    chart1: '--chart-1',
    chart2: '--chart-2',
    chart3: '--chart-3',
    chart4: '--chart-4',
    chart5: '--chart-5',
    sidebar: '--sidebar',
    sidebarForeground: '--sidebar-foreground',
    sidebarPrimary: '--sidebar-primary',
    sidebarPrimaryForeground: '--sidebar-primary-foreground',
    sidebarAccent: '--sidebar-accent',
    sidebarAccentForeground: '--sidebar-accent-foreground',
    sidebarBorder: '--sidebar-border',
    sidebarRing: '--sidebar-ring',
  };

  // Apply custom theme colors to CSS variables
  function applyCustomThemeColors(colors: Record<string, string>) {
    const root = document.documentElement;
    for (const [key, cssVar] of Object.entries(customThemeKeyToCssVar)) {
      const colorValue = colors[key]
      if (colorValue) {
        root.style.setProperty(cssVar, colorValue);
      }
    }
  }

  // Remove custom theme inline styles
  function clearCustomThemeColors() {
    const root = document.documentElement;
    for (const cssVar of Object.values(customThemeKeyToCssVar)) {
      root.style.removeProperty(cssVar);
    }
  }

</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

<!-- Apply theme on the document element -->
  {(() => {
  $effect(() => {
    if (typeof document !== 'undefined') {
      const customThemeColors = SETTINGS.accessibility.state.customThemeColors;

      // Always operate in 'custom' theme mode. Apply any custom colors if present.
      document.documentElement.setAttribute('data-theme', 'custom');

      if (customThemeColors) {
        applyCustomThemeColors(customThemeColors);
      } else {
        clearCustomThemeColors();
      }
    }
  });
})()}

{(() => {
  $effect(() => {
    if (typeof window === "undefined") return;
    const bossOnlyEnabled = Boolean(SETTINGS.live.general.state.bossOnlyDps);
    if (lastBossOnlySync === bossOnlyEnabled) return;
    lastBossOnlySync = bossOnlyEnabled;
    void setBossOnlyDps(bossOnlyEnabled);
  });

  $effect(() => {
    if (typeof window === "undefined") return;
    const segmentsEnabled = Boolean(SETTINGS.live.general.state.dungeonSegmentsEnabled);
    if (lastDungeonSegmentsSync === segmentsEnabled) return;
    lastDungeonSegmentsSync = segmentsEnabled;
    void setDungeonSegmentsEnabled(segmentsEnabled);
  });
})()}

{@render children()}
