<script lang="ts">
  let {
    label = "",
    description = "",
    value = $bindable("rgba(255, 255, 255, 1)"),
    oninput,
  }: {
    label: string;
    description?: string | undefined;
    value: string;
    oninput?: (value: string) => void;
  } = $props();

  // Parse rgba string to components
  function parseRgba(rgba: string): { r: number; g: number; b: number; a: number } {
    const match = rgba.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
    if (match) {
      return {
        r: parseInt(match[1] ?? '255', 10),
        g: parseInt(match[2] ?? '255', 10),
        b: parseInt(match[3] ?? '255', 10),
        a: match[4] !== undefined ? parseFloat(match[4]) : 1
      };
    }
    // Fallback for hex
    if (rgba.startsWith('#')) {
      const hex = rgba.slice(1);
      if (hex.length === 6) {
        return {
          r: parseInt(hex.slice(0, 2), 16),
          g: parseInt(hex.slice(2, 4), 16),
          b: parseInt(hex.slice(4, 6), 16),
          a: 1
        };
      } else if (hex.length === 8) {
        return {
          r: parseInt(hex.slice(0, 2), 16),
          g: parseInt(hex.slice(2, 4), 16),
          b: parseInt(hex.slice(4, 6), 16),
          a: parseInt(hex.slice(6, 8), 16) / 255
        };
      }
    }
    return { r: 255, g: 255, b: 255, a: 1 };
  }

  // Convert components to rgba string
  function toRgba(r: number, g: number, b: number, a: number): string {
    return `rgba(${r}, ${g}, ${b}, ${a})`;
  }

  // Convert to hex for color input
  function toHex(r: number, g: number, b: number): string {
    return '#' + [r, g, b].map(x => x.toString(16).padStart(2, '0')).join('');
  }

  let parsed = $derived(parseRgba(value));
  let hexColor = $derived(toHex(parsed.r, parsed.g, parsed.b));
  let alphaPercent = $derived(Math.round(parsed.a * 100));

  function handleColorChange(e: Event) {
    const hex = (e.target as HTMLInputElement).value;
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    const newValue = toRgba(r, g, b, parsed.a);
    value = newValue;
    oninput?.(newValue);
  }

  function handleAlphaChange(e: Event) {
    const alpha = parseInt((e.target as HTMLInputElement).value, 10) / 100;
    const newValue = toRgba(parsed.r, parsed.g, parsed.b, Math.round(alpha * 100) / 100);
    value = newValue;
    oninput?.(newValue);
  }
</script>

<label class="flex items-center justify-between gap-3 py-2.5 px-3 rounded-md hover:bg-popover/50 transition-colors">
  <div class="flex flex-col gap-0.5 min-w-0 flex-1">
    <div class="text-sm font-medium text-foreground">{label}</div>
    {#if description}
      <div class="text-xs text-muted-foreground leading-relaxed">{description}</div>
    {/if}
  </div>
  <div class="flex items-center gap-2 flex-shrink-0">
    <input
      type="color"
      value={hexColor}
      oninput={handleColorChange}
      class="w-8 h-8 rounded cursor-pointer border border-border/50 bg-transparent"
    />
    <div class="flex items-center gap-1">
      <input
        type="range"
        min="0"
        max="100"
        value={alphaPercent}
        oninput={handleAlphaChange}
        class="w-16 h-2 accent-primary cursor-pointer"
      />
      <span class="text-xs text-muted-foreground font-mono w-8">{alphaPercent}%</span>
    </div>
  </div>
</label>
