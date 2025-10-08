<script lang="ts">
  import { Switch } from "$lib/components/ui/switch/index.js";

  let {
    label = "",
    description = "",
    checked = $bindable(false),
  }: {
    label: string;
    description?: string | undefined;
    checked: boolean | undefined;
  } = $props();

  // If checked is undefined, give it a default (e.g. false)
  if (checked === undefined) {
    checked = true;
  }
</script>

<label class="flex flex-row items-center">
  <Switch bind:checked />
  <div class="ml-4">
    <div>{label}</div>
    {#if description}
      <div class="text-muted-foreground text-sm">{description}</div>
    {/if}
  </div>
</label>
