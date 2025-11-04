<script lang="ts">
  /**
   * @file This component displays a glowing effect on a table row, based on the player's class and a percentage value.
   */
  import { getClassColor } from '$lib/utils.svelte';

  let {
    className,
    percentage
  }: {
    className: string;
    percentage: number;
  } = $props();

  let classColor = $derived(getClassColor(className));
</script>

<!-- Gradient glow from bottom -->
<td
  class="absolute left-0 bottom-0 h-full pointer-events-none"
  style="background: linear-gradient(to top, {classColor}, transparent), linear-gradient(to right, {classColor} 0%, {classColor} 70%, transparent 100%); width: {percentage}%; opacity: 0.15;"
></td>
<!-- Neon border bottom effect -->
<td
  class="absolute left-0 bottom-0 h-[2px] pointer-events-none z-20"
  style="background-color: {classColor}; width: {percentage}%; box-shadow: 0 0 4px {classColor}, 0 0 8px {classColor};"
></td>
