<script lang="ts">
  /**
   * @file This component displays a number in an abbreviated format.
   */
  let {
    num = 0,
    suffixFontSize,
    suffixColor,
  }: {
    num: number;
    suffixFontSize?: number;
    suffixColor?: string;
  } = $props();

  function abbreviateNumberSplit(n: number): [string, string] {
    if (n >= 1e3 && n < 1e6) return [(n / 1e3).toFixed(1), "k"];
    if (n >= 1e6 && n < 1e9) return [(n / 1e6).toFixed(1), "m"];
    if (n >= 1e9 && n < 1e12) return [(n / 1e9).toFixed(1), "b"];
    if (n >= 1e12) return [(n / 1e12).toFixed(1), "t"];
    else return [n.toFixed(0), ""];
  }

  let abbreviatedNumberTuple = $derived(abbreviateNumberSplit(num));
  let fullNumberString = $derived(num.toLocaleString());

  let suffixStyle = $derived(
    [
      suffixFontSize ? `font-size: ${suffixFontSize}px` : '',
      suffixColor ? `color: ${suffixColor}` : ''
    ].filter(Boolean).join('; ')
  );
</script>

<span title={fullNumberString} class="whitespace-nowrap inline-flex items-baseline gap-0.5">
  {abbreviatedNumberTuple[0]}<span class="text-tiny text-muted-foreground" style={suffixStyle || undefined}>{abbreviatedNumberTuple[1]}</span>
</span>
