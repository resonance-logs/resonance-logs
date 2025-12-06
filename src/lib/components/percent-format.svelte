<script lang="ts">
  /**
   * @file This component displays a number as a percentage.
   */
  let {
    val = "",
    fractionDigits = 0,
    suffixFontSize,
    suffixColor,
  }: {
    val: string | number;
    fractionDigits?: number;
    suffixFontSize?: number;
    suffixColor?: string;
  } = $props();

  let suffixStyle = $derived(
    [
      suffixFontSize ? `font-size: ${suffixFontSize}px` : '',
      suffixColor ? `color: ${suffixColor}` : ''
    ].filter(Boolean).join('; ')
  );
</script>

<span class="whitespace-nowrap inline-flex items-baseline gap-1">
  {typeof val === "number" ? val.toFixed(fractionDigits) : val}
  <span class="text-tiny text-muted-foreground" style={suffixStyle || undefined}>%</span>
</span>
