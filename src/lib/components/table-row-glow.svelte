<script lang="ts">
  import { getClassColor } from '$lib/utils.svelte';
  import { SETTINGS } from '$lib/settings-store';

  let {
    className,
    classSpecName = "",
    percentage
  }: {
    className: string;
    classSpecName?: string;
    percentage: number;
  } = $props();

  let classColor = $derived(getClassColor(className, classSpecName));

  // derive customization from live table settings using runes-friendly $derived
  // glow is always enabled and always uses class/spec color
  let mode = $derived(SETTINGS.live.tableCustomization.state.rowGlowMode);
  let opacity = $derived(SETTINGS.live.tableCustomization.state.rowGlowOpacity);
  let borderHeight = $derived(SETTINGS.live.tableCustomization.state.rowGlowBorderHeight);
  let spread = $derived(SETTINGS.live.tableCustomization.state.rowGlowSpread);
  let rowBorderRadius = $derived(SETTINGS.live.tableCustomization.state.rowBorderRadius);
  let rowBorderWidth = $derived(SETTINGS.live.tableCustomization.state.rowBorderWidth);
  let rowBorderColor = $derived(SETTINGS.live.tableCustomization.state.rowBorderColor);

  // glowColor is always the class/spec color
  let glowColor = $derived.by(() => classColor);
</script>
{#if mode === 'solid'}
  <!-- Solid full-color fill (no gradient, no underline) -->
  <td
    class="absolute left-0 bottom-0 top-0 pointer-events-none"
    style="background-color: {glowColor}; width: {percentage}%; opacity: {opacity}; border-radius: {rowBorderRadius}px; {rowBorderWidth > 0 ? `box-shadow: inset 0 0 0 ${rowBorderWidth}px ${rowBorderColor};` : ''}"
  ></td>
{:else if mode === 'gradient'}
  <!-- Gradient fill only -->
  <td
    class="absolute left-0 bottom-0 h-full pointer-events-none"
    style="background: linear-gradient(to top, {glowColor}, transparent), linear-gradient(to right, {glowColor} 0%, {glowColor} 70%, transparent 100%); width: {percentage}%; opacity: {opacity}; border-radius: {rowBorderRadius}px; {rowBorderWidth > 0 ? `box-shadow: inset 0 0 0 ${rowBorderWidth}px ${rowBorderColor};` : ''}"
  ></td>
{:else}
  <!-- gradient-underline: gradient fill with neon underline -->
  <td
    class="absolute left-0 bottom-0 h-full pointer-events-none"
    style="background: linear-gradient(to top, {glowColor}, transparent), linear-gradient(to right, {glowColor} 0%, {glowColor} 70%, transparent 100%); width: {percentage}%; opacity: {opacity}; border-radius: {rowBorderRadius}px; {rowBorderWidth > 0 ? `box-shadow: inset 0 0 0 ${rowBorderWidth}px ${rowBorderColor};` : ''}"
  ></td>
  <td
    class="absolute left-0 bottom-0 pointer-events-none z-20"
    style="height: {borderHeight}px; background-color: {glowColor}; width: {percentage}%; box-shadow: 0 0 {Math.max(2, spread/2)}px {glowColor}, 0 0 {spread}px {glowColor}{rowBorderWidth > 0 ? `, inset 0 0 0 ${rowBorderWidth}px ${rowBorderColor}` : ''}; border-radius: 0 0 {rowBorderRadius}px {rowBorderRadius}px;"
  ></td>
{/if}
