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

{typeof val === "number" ? val.toFixed(fractionDigits) : val}<span class="text-tiny text-gray-300">%</span>
