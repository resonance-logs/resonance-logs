<script lang="ts">
  /**
   * @file This component displays a number as a percentage.
   */
  let {
    val = "",
    fractionDigits = 0,
  }: {
    val: string | number;
    fractionDigits?: number;
  } = $props();
</script>

<span class="whitespace-nowrap inline-flex items-baseline gap-1">
  {typeof val === "number" ? val.toFixed(fractionDigits) : val}
  <span class="text-tiny text-gray-300">%</span>
</span>
