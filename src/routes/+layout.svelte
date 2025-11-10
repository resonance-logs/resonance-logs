<script lang="ts">
  /**
   * @file This is the root layout for the application.
   * It imports the global stylesheet and disables the context menu.
   */
  import "../app.css";
  import { SETTINGS } from "$lib/settings-store";

  let { children } = $props();
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

<!-- Apply theme on the document element -->
{(() => {
  // Derive theme with fallback
  const theme = $derived(SETTINGS.accessibility.state.theme ?? 'dark');
  // Derive transparent mode and mirror into body class so window background becomes transparent
  const transparentMode = $derived(SETTINGS.accessibility.state.transparentMode ?? false);
  $effect(() => {
    if (typeof document !== 'undefined') {
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

{@render children()}
