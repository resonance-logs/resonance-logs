<script lang="ts">
  /**
   * @file This is the root layout for the application.
   * It imports the global stylesheet and disables the context menu.
   */
  import "../app.css";
  import { SETTINGS, DEFAULT_CUSTOM_THEME_COLORS } from "$lib/settings-store";
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

  // Mapping from camelCase keys to CSS variable names
  const customThemeKeyToCssVar: Record<string, string> = {
    background: '--background',
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
      const colorValue = colors[key] ?? DEFAULT_CUSTOM_THEME_COLORS[key];
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
      const theme = SETTINGS.accessibility.state.theme ?? 'dark';
      const transparentMode = SETTINGS.accessibility.state.transparentMode ?? false;
      const customThemeColors = SETTINGS.accessibility.state.customThemeColors;
      
      document.documentElement.setAttribute('data-theme', theme);
      
      // Apply or clear custom theme colors based on selected theme
      if (theme === 'custom' && customThemeColors) {
        applyCustomThemeColors(customThemeColors);
      } else {
        clearCustomThemeColors();
      }
      
      try {
        // Mirror into localStorage for early load in app.html script
        const raw = localStorage.getItem('accessibility');
        const parsed = raw ? JSON.parse(raw) : {};
        parsed.theme = theme;
        localStorage.setItem('accessibility', JSON.stringify(parsed));
      } catch {}

      // Mirror transparent mode into body class so window background becomes transparent
      if (transparentMode) {
        document.body.classList.add('transparent-mode');
      } else {
        document.body.classList.remove('transparent-mode');
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
