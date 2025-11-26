<script lang="ts">
  /**
   * @file Settings file picker component for importing files (images, fonts)
   */
  import UploadIcon from "virtual:icons/lucide/upload";
  import XIcon from "virtual:icons/lucide/x";

  interface Props {
    label: string;
    description?: string;
    value: string;
    accept: string;
    onchange: (dataUrl: string, fileName: string) => void;
    onclear: () => void;
  }

  let { label, description, value, accept, onchange, onclear }: Props = $props();

  let fileInput: HTMLInputElement;
  let fileName = $state('');
  let isLoading = $state(false);

  // Extract filename from value if it's a data URL with name
  $effect(() => {
    if (!value) {
      fileName = '';
    }
  });

  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    isLoading = true;
    fileName = file.name;
    
    const reader = new FileReader();
    reader.onload = (e) => {
      const dataUrl = e.target?.result as string;
      onchange(dataUrl, file.name);
      isLoading = false;
    };
    reader.onerror = () => {
      console.error('Failed to read file');
      isLoading = false;
    };
    reader.readAsDataURL(file);
  }

  function handleClear() {
    fileName = '';
    if (fileInput) {
      fileInput.value = '';
    }
    onclear();
  }
</script>

<div class="flex items-center justify-between py-2.5 px-3 min-h-[48px] rounded-lg hover:bg-popover/40 transition-colors gap-4">
  <div class="flex flex-col min-w-0 flex-1">
    <span class="text-sm font-medium text-foreground">{label}</span>
    {#if description}
      <span class="text-xs text-muted-foreground mt-0.5">{description}</span>
    {/if}
  </div>
  
  <div class="flex items-center gap-2 shrink-0">
    {#if value || fileName}
      <span class="text-xs text-muted-foreground max-w-[150px] truncate">
        {fileName || 'File loaded'}
      </span>
      <button
        type="button"
        class="p-1.5 rounded-md hover:bg-destructive/20 text-muted-foreground hover:text-destructive transition-colors"
        onclick={handleClear}
        title="Clear"
      >
        <XIcon class="w-4 h-4" />
      </button>
    {/if}
    
    <label class="flex items-center gap-2 px-3 py-1.5 rounded-md bg-muted hover:bg-muted/80 text-foreground text-sm font-medium cursor-pointer transition-colors">
      <UploadIcon class="w-4 h-4" />
      <span>{value || fileName ? 'Change' : 'Browse'}</span>
      <input
        bind:this={fileInput}
        type="file"
        {accept}
        class="hidden"
        onchange={handleFileSelect}
        disabled={isLoading}
      />
    </label>
  </div>
</div>
