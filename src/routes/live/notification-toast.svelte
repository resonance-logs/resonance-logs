<script lang="ts">
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
    switch (type) {
      case 'error':
        return 'bg-red-900/40 border-red-500/50';
      case 'notice':
        return 'bg-neutral-800/60 border-neutral-600/50';
    }
  }
</script>

<!-- Toast container positioned at bottom center -->
<div class="pointer-events-none fixed bottom-10 left-0 right-0 z-50 flex flex-col items-center gap-2 px-4">
  {#each toasts as toast (toast.id)}
    <div
      in:fly={{ y: 20, duration: 300 }}
      out:fade={{ duration: 200 }}
      class={`pointer-events-auto flex items-center gap-2 rounded border px-3 py-2 text-xs text-white shadow-xl backdrop-blur-md ${getToastClass(toast.type)}`}
    >
      <span>{toast.message}</span>
      <button
        onclick={() => dismissToast(toast.id)}
        class="ml-2 rounded-sm p-0.5 hover:bg-white/10 transition-colors"
        aria-label="Dismiss"
      >
        <XIcon class="h-4 w-4" />
      </button>
    </div>
  {/each}
</div>
