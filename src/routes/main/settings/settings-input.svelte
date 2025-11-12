<script lang="ts">
  let {
    label = "",
    description = "",
    placeholder = "",
    type = "text",
    value = $bindable(""),
  }: {
    label: string;
    description?: string | undefined;
    placeholder?: string;
    type?: "text" | "password" | "url" | "number";
    value: string | number;
  } = $props();
</script>

<label class="flex flex-col gap-1.5 py-2.5 px-3 rounded-md hover:bg-popover/50 transition-colors">
  <div class="flex flex-col gap-0.5">
    <div class="text-sm font-medium text-foreground">{label}</div>
    {#if description}
      <div class="text-xs text-muted-foreground leading-relaxed">{description}</div>
    {/if}
  </div>
  <input
    {type}
    bind:value
    {placeholder}
    class="w-full px-3 py-2 text-sm bg-popover border border-border rounded-md
           focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary
           placeholder:text-muted-foreground/50
           transition-colors"
  />
</label>