<script lang="ts">
  /**
   * @file This component displays notification toasts.
   */
  import { fly, fade } from "svelte/transition";
  import XIcon from "virtual:icons/lucide/x";

  type ToastType = 'error' | 'notice';

  interface Toast {
    id: number;
    type: ToastType;
    message: string;
  }

  let toasts = $state<Toast[]>([]);
  let nextId = 0;

  export function showToast(type: ToastType, message: string) {
    const id = nextId++;
    toasts.push({ id, type, message });

    // Auto-dismiss after 3 seconds
    setTimeout(() => {
      dismissToast(id);
    }, 3000);
  }

  function dismissToast(id: number) {
    toasts = toasts.filter(t => t.id !== id);
  }

  function getToastClass(type: ToastType): string {
    // Use semantic tokens so themes can override without editing component.
    // Provide subtle hue for error while keeping readability.
    switch (type) {
      case 'error':
        return 'bg-destructive/25 border border-destructive/40 text-destructive-foreground shadow-[0_0_0_1px_var(--border)]';
      case 'notice':
        return 'bg-popover/60 border border-border/60 text-foreground shadow-[0_0_0_1px_var(--border)]';
    }
  }
</script>

<!-- Toast container positioned at bottom center -->
<div class="pointer-events-none fixed bottom-12 left-0 right-0 z-50 flex flex-col items-center gap-2 px-4">
  {#each toasts as toast (toast.id)}
    <div
      in:fly={{ y: 40, duration: 260 }}
      out:fade={{ duration: 160 }}
      class={`pointer-events-auto group flex items-center gap-2 rounded-md px-3 py-2 text-xs leading-tight backdrop-blur-sm transition-colors duration-200 ${getToastClass(toast.type)}`}
    >
      <span class="select-none font-medium tracking-tight">{toast.message}</span>
      <button
        onclick={() => dismissToast(toast.id)}
        class="ml-1 rounded-sm p-0.5 text-muted-foreground hover:text-foreground hover:bg-muted/30 transition-colors"
        aria-label="Dismiss"
      >
        <XIcon class="h-4 w-4" />
      </button>
    </div>
  {/each}
</div>
