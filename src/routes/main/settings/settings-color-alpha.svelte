<script lang="ts">
  import Color from 'colorjs.io';

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

  // Parse any color format (oklch, rgba, hex, etc.) to RGBA using colorjs.io
  function parseToRgba(colorStr: string): { r: number; g: number; b: number; a: number } {
    try {
      const color = new Color(colorStr);
      const srgb = color.to('srgb');
      const r = Math.round(Math.max(0, Math.min(1, srgb.coords[0] ?? 0)) * 255);
      const g = Math.round(Math.max(0, Math.min(1, srgb.coords[1] ?? 0)) * 255);
      const b = Math.round(Math.max(0, Math.min(1, srgb.coords[2] ?? 0)) * 255);
      const a = srgb.alpha ?? 1;
      return { r, g, b, a };
    } catch {
      return { r: 255, g: 255, b: 255, a: 1 };
    }
  }

  // Convert to hex for color input
  function toHex(r: number, g: number, b: number): string {
    const clamp = (n: number) => Math.max(0, Math.min(255, Math.round(n)));
    return '#' + [clamp(r), clamp(g), clamp(b)].map(x => x.toString(16).padStart(2, '0')).join('');
  }

  // Local state - initialized once from value, then managed locally
  const initial = parseToRgba(value);
  let r = $state(initial.r);
  let g = $state(initial.g);
  let b = $state(initial.b);
  let a = $state(initial.a);
  
  // Track external value changes
  let lastExternalValue = value;
  
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  const DEBOUNCE_MS = 10;
  
  $effect(() => {
    if (value !== lastExternalValue) {
      const parsed = parseToRgba(value);
      r = parsed.r;
      g = parsed.g;
      b = parsed.b;
      a = parsed.a;
      lastExternalValue = value;
    }
  });

  let hexColor = $derived(toHex(r, g, b));
  let alphaPercent = $derived(Math.round(a * 100));

  function emitValue(newR: number, newG: number, newB: number, newA: number) {
    const newValue = `rgba(${newR}, ${newG}, ${newB}, ${newA})`;
    lastExternalValue = newValue;
    
    // Clear any pending debounce
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    
    debounceTimer = setTimeout(() => {
      value = newValue;
      oninput?.(newValue);
      debounceTimer = null;
    }, DEBOUNCE_MS);
  }

  function handleColorChange(e: Event) {
    const hex = (e.target as HTMLInputElement).value;
    r = parseInt(hex.slice(1, 3), 16);
    g = parseInt(hex.slice(3, 5), 16);
    b = parseInt(hex.slice(5, 7), 16);
    emitValue(r, g, b, a);
  }

  function handleAlphaChange(e: Event) {
    a = Math.round(parseInt((e.target as HTMLInputElement).value, 10)) / 100;
    emitValue(r, g, b, a);
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
