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

  console.log(value, label)

  // Utility: clamp number
  function clamp01(v: number) {
    return Math.min(1, Math.max(0, v));
  }

  // Convert linear sRGB to gamma-corrected sRGB
  function linearToSRGB(c: number) {
    if (c <= 0.0031308) return 12.92 * c;
    return 1.055 * Math.pow(c, 1 / 2.4) - 0.055;
  }

  // Convert OKLab -> linear sRGB -> sRGB
  function oklabToSRGB(L: number, a: number, b: number) {
    // From OKLab spec
    const l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    const m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    const s_ = L - 0.0894841775 * a - 1.2914855480 * b;

    const l = l_ * l_ * l_;
    const m = m_ * m_ * m_;
    const s = s_ * s_ * s_;

    let r = +4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let bb = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    r = clamp01(linearToSRGB(r));
    g = clamp01(linearToSRGB(g));
    bb = clamp01(linearToSRGB(bb));

    return {
      r: Math.round(r * 255),
      g: Math.round(g * 255),
      b: Math.round(bb * 255),
    };
  }

  // Parse OKLCH string and convert to rgba components
  function parseOklch(oklch: string): { r: number; g: number; b: number; a: number } | null {
    const m = oklch.match(/oklch\(([^)]+)\)/i);
    if (!m) return null;
    const partsRaw = (m[1] ?? '').trim();
    // handle slash alpha syntax: "L C H / a"
    const parts = partsRaw.split('/').map(p => p.trim());
    let alpha = 1;
    if (parts.length === 2) {
      const alphaPart = parts[1] ?? '';
      if (alphaPart !== '') {
        const parsedAlpha = parseFloat(alphaPart);
        if (!Number.isNaN(parsedAlpha)) alpha = parsedAlpha;
      }
    }
    const main = (parts[0] ?? '').split(/\s+/).filter(Boolean);
    const L = parseFloat(main[0] ?? '0');
    const C = parseFloat(main[1] ?? '0');
    const Hraw = main[2] ?? '0';
    // H may include 'deg'
    let h = parseFloat(Hraw.replace('deg', ''));
    if (Number.isNaN(h)) h = 0;
    // If h looks like degrees (greater than ~6.3 radians), convert to radians
    if (h > 6.3) {
      h = (h * Math.PI) / 180;
    }
    const a = C * Math.cos(h);
    const b = C * Math.sin(h);

    return { ...oklabToSRGB(L, a, b), a: clamp01(alpha) };
  }

  // Parse various color formats (rgba(), rgb(), hex, oklch())
  function parseColor(col: string): { r: number; g: number; b: number; a: number } {
    if (!col) return { r: 255, g: 255, b: 255, a: 1 };
    col = col.trim();
    // OKLCH
    if (/^oklch\(/i.test(col)) {
      const parsed = parseOklch(col);
      if (parsed) return parsed;
      return { r: 255, g: 255, b: 255, a: 1 };
    }
    // rgba / rgb
    const m = col.match(/rgba?\(([^)]+)\)/i);
    if (m) {
      const content = m[1] ?? '';
      const parts = content.split(',').map(p => p.trim());
      const r = parseInt(parts[0] ?? '255', 10) || 255;
      const g = parseInt(parts[1] ?? '255', 10) || 255;
      const b = parseInt(parts[2] ?? '255', 10) || 255;
      const a = parts[3] !== undefined ? parseFloat(parts[3] ?? '1') : 1;
      return { r, g, b, a };
    }
    // hex
    if (col.startsWith('#')) {
      const hex = col.slice(1);
      if (hex.length === 6) {
        return {
          r: parseInt(hex.slice(0, 2), 16),
          g: parseInt(hex.slice(2, 4), 16),
          b: parseInt(hex.slice(4, 6), 16),
          a: 1,
        };
      }
      if (hex.length === 8) {
        return {
          r: parseInt(hex.slice(0, 2), 16),
          g: parseInt(hex.slice(2, 4), 16),
          b: parseInt(hex.slice(4, 6), 16),
          a: parseInt(hex.slice(6, 8), 16) / 255,
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
    const clamp = (n: number) => Math.max(0, Math.min(255, Math.round(n)));
    return (
      '#' + [clamp(r), clamp(g), clamp(b)].map(x => x.toString(16).padStart(2, '0')).join('')
    );
  }

  let parsed = $derived(parseColor(value));
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
