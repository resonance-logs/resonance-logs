<script lang="ts">
  import { Slider } from "$lib/components/ui/slider/index.js";

  let {
    label = "",
    description = "",
    value = $bindable(60),
    unit = "%",
    min = 0,
    max = 100,
    step = 5,
    ...restProps
  }: {
    label: string;
    description?: string | undefined;
    value: number;
    unit?: string;
    min?: number;
    max?: number;
    step?: number;
    [key: string]: any;
  } = $props();
</script>

<label class="items-center">
  <div class="mb-4">
    <div>{label}</div>
    {#if description}
      <div class="text-muted-foreground text-sm">{description}</div>
    {/if}
  </div>
  <div class="flex flex-row">
    <Slider
      type="single"
      bind:value
      {min}
      {max}
      {step}
      class="max-w-[70%]"
      {...restProps}
    />
    <div class="ml-4">
      <span>{value}{unit}</span>
    </div>
  </div>
</label>
