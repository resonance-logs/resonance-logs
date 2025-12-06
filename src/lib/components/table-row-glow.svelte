<script lang="ts">
  import { getClassColor } from '$lib/utils.svelte';
  import { SETTINGS } from '$lib/settings-store';

  let {
    className,
    classSpecName = "",
    percentage,
    isSkill = false
  }: {
    className: string;
    classSpecName?: string;
    percentage: number;
    isSkill?: boolean;
  } = $props();

  let classColor = $derived(getClassColor(className, classSpecName));

  // derive customization from live table settings using runes-friendly $derived
  // Choose skill-specific settings when rendering skill rows
  let mode = $derived.by(() => isSkill ? SETTINGS.live.tableCustomization.state.skillRowGlowMode : SETTINGS.live.tableCustomization.state.rowGlowMode);
  let opacity = $derived.by(() => isSkill ? SETTINGS.live.tableCustomization.state.skillRowGlowOpacity : SETTINGS.live.tableCustomization.state.rowGlowOpacity);
  let borderHeight = $derived(SETTINGS.live.tableCustomization.state.rowGlowBorderHeight);
  let spread = $derived(SETTINGS.live.tableCustomization.state.rowGlowSpread);
  let rowBorderRadius = $derived.by(() => isSkill ? SETTINGS.live.tableCustomization.state.skillRowBorderRadius : SETTINGS.live.tableCustomization.state.rowBorderRadius);
  

  // glowColor is always the class/spec color
  let glowColor = $derived.by(() => classColor);
</script>
{#if mode === 'solid'}
  <!-- Solid full-color fill (no gradient, no underline) -->
  <td
    class="absolute left-0 bottom-0 top-0 pointer-events-none"
    style="background-color: {glowColor}; width: {percentage}%; opacity: {opacity}; border-radius: {rowBorderRadius}px;"
  ></td>
{:else if mode === 'gradient'}
  <!-- Gradient fill only -->
  <td
    class="absolute left-0 bottom-0 h-full pointer-events-none"
    style="background: linear-gradient(to top, {glowColor}, transparent), linear-gradient(to right, {glowColor} 0%, {glowColor} 70%, transparent 100%); width: {percentage}%; opacity: {opacity}; border-radius: {rowBorderRadius}px;"
  ></td>
{:else}
  <!-- gradient-underline: gradient fill with neon underline -->
  <td
    class="absolute left-0 bottom-0 h-full pointer-events-none"
    style="background: linear-gradient(to top, {glowColor}, transparent), linear-gradient(to right, {glowColor} 0%, {glowColor} 70%, transparent 100%); width: {percentage}%; opacity: {opacity}; border-radius: {rowBorderRadius}px;"
  ></td>
  <td
    class="absolute left-0 bottom-0 pointer-events-none z-20"
    style="height: {borderHeight}px; background-color: {glowColor}; width: {percentage}%; box-shadow: 0 0 {Math.max(2, spread/2)}px {glowColor}, 0 0 {spread}px {glowColor}; border-radius: 0 0 {rowBorderRadius}px {rowBorderRadius}px;"
  ></td>
{/if}
