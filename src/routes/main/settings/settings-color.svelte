<script lang="ts">
  let {
    label = "",
    description = "",
    value = $bindable("#ffffff"),
  }: {
    label: string;
    description?: string | undefined;
    value: string;
  } = $props();
</script>

<label class="flex items-center justify-between gap-3 py-2.5 px-3 rounded-md hover:bg-popover/50 transition-colors">
  <div class="flex flex-col gap-0.5">
    <div class="text-sm font-medium text-foreground">{label}</div>
    {#if description}
      <div class="text-xs text-muted-foreground leading-relaxed">{description}</div>
    {/if}
  </div>
  <div class="flex items-center gap-2">
    <input
      type="color"
      bind:value
      class="w-8 h-8 rounded cursor-pointer border border-border/50 bg-transparent"
    />
    <span class="text-xs text-muted-foreground font-mono w-16">{value}</span>
  </div>
</label>
