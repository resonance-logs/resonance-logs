<script lang="ts">
  /**
   * @file This is the root layout for the application.
   * It imports the global stylesheet and disables the context menu.
   */
  import "../app.css";

  let { children } = $props();
</script>

<svelte:window oncontextmenu={(e) => e.preventDefault()} />

{@render children()}
