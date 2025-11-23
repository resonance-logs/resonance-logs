<script lang="ts">
  /**
   * @file This is the root layout for the application.
   * It imports the global stylesheet and disables the context menu.
   */
  import "../app.css";
  import { SETTINGS } from "$lib/settings-store";
  import { commands } from "$lib/bindings";
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
})()}

{(() => {
  $effect(() => {
    if (typeof window === "undefined") return;
    const enabled = SETTINGS.moduleSync.state.enabled;
    const apiKey = (SETTINGS.moduleSync.state.apiKey || "").trim();
    let baseUrl = (SETTINGS.moduleSync.state.baseUrl || "").trim();
    const interval = SETTINGS.moduleSync.state.autoSyncIntervalMinutes || 0;

    const LEGACY_BASE_URL = "http://localhost:8080/api/v1";
    const DEFAULT_BASE_URL = "https://api.bpsr.app/api/v1";

    if (!baseUrl || baseUrl === LEGACY_BASE_URL) {
      if (SETTINGS.moduleSync.state.baseUrl !== DEFAULT_BASE_URL) {
        SETTINGS.moduleSync.state.baseUrl = DEFAULT_BASE_URL;
      }
      baseUrl = DEFAULT_BASE_URL;
    }

    commands
      .setModuleSyncConfig(enabled, apiKey || null, baseUrl || null, interval)
      .catch(() => {});
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
