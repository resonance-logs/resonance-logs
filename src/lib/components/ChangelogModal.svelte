<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { marked } from 'marked';
  import changelogRaw from '../../../CHANGELOG.md?raw';

  const dispatch = createEventDispatcher();

  let html = $state('');

  onMount(async () => {
    try {
      // Parse markdown asynchronously
      html = await marked.parse(changelogRaw) as string;
    } catch (err) {
      console.error('Failed to parse changelog:', err);
      html = `<pre>${changelogRaw}</pre>`;
    }
  });

  function close() {
    dispatch('close');
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center">
  <!-- Backdrop -->
  <button
    class="absolute inset-0 bg-black/60 backdrop-blur-[2px]"
    onclick={close}
    type="button"
    aria-label="Close modal"
  >
  </button>

  <div class="relative bg-card border border-border rounded-xl shadow-2xl w-[90vw] max-w-4xl h-[85vh] overflow-hidden z-10 flex flex-col">
    <div class="flex items-center justify-between px-6 py-4 border-b border-border">
      <h2 class="text-xl font-semibold">Changelog</h2>
      <button
        class="text-muted-foreground hover:text-foreground transition-colors p-2 hover:bg-muted rounded-md"
        type="button"
        onclick={close}
        aria-label="Close"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
    <div class="flex-1 overflow-auto p-6">
      {#if html}
        <div class="space-y-2 prose dark:prose-invert max-w-none">
          {@html html}
        </div>
      {:else}
        <div class="flex items-center justify-center h-full text-muted-foreground">
          Loading changelog...
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  :global(.prose p) { margin: 0 0 0.75rem 0; }
  :global(.prose ul) { padding-left: 1.25rem; margin: 0 0 0.75rem 0; }
  :global(.prose h1) { font-size: 1.25rem; margin-bottom: 0.5rem; }
  :global(.prose h2) { font-size: 1.125rem; margin-bottom: 0.4rem; }
  :global(.prose h3) { font-size: 1rem; margin-bottom: 0.3rem; }
</style>
