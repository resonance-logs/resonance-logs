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
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

<!-- Apply theme on the document element -->
  {(() => {
  $effect(() => {
    if (typeof document !== 'undefined') {
      const theme = SETTINGS.accessibility.state.theme || 'dark';
      
      // Set the data-theme attribute based on the saved theme setting
      document.documentElement.setAttribute('data-theme', theme);
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
