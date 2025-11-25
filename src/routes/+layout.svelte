<script lang="ts">
  /**
   * @file This is the root layout for the application.
   * It imports the global stylesheet and disables the context menu.
   */
  import "../app.css";
  import { SETTINGS, DEFAULT_FONT_SIZES } from "$lib/settings-store";
  import { setBossOnlyDps, setDungeonSegmentsEnabled } from "$lib/api";

  let { children } = $props();
  let lastBossOnlySync: boolean | null = null;
  let lastDungeonSegmentsSync: boolean | null = null;
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

<!-- Apply theme on the document element -->
{(() => {
  $effect(() => {
    if (typeof document !== 'undefined') {
      const theme = SETTINGS.accessibility.state.theme ?? 'dark';
      const transparentMode = SETTINGS.accessibility.state.transparentMode ?? false;
      document.documentElement.setAttribute('data-theme', theme);
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

  // Apply custom font sizes to CSS variables
  $effect(() => {
    if (typeof document !== 'undefined') {
      const fontSizes = SETTINGS.accessibility.state.fontSizes ?? DEFAULT_FONT_SIZES;
      const root = document.documentElement;
      root.style.setProperty('--font-size-xs', `${fontSizes.xs ?? DEFAULT_FONT_SIZES.xs}px`);
      root.style.setProperty('--font-size-sm', `${fontSizes.sm ?? DEFAULT_FONT_SIZES.sm}px`);
      root.style.setProperty('--font-size-base', `${fontSizes.base ?? DEFAULT_FONT_SIZES.base}px`);
      root.style.setProperty('--font-size-lg', `${fontSizes.lg ?? DEFAULT_FONT_SIZES.lg}px`);
      root.style.setProperty('--font-size-xl', `${fontSizes.xl ?? DEFAULT_FONT_SIZES.xl}px`);
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
