<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSelect from "./settings-select.svelte";
  import { SETTINGS, AVAILABLE_THEMES } from "$lib/settings-store";

  const SETTINGS_CATEGORY = "themes";
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <div class="space-y-3">
    <div class="bg-popover/40 rounded-lg border border-border/50 p-4 space-y-3">
      <div>
        <h2 class="text-base font-semibold text-foreground">Color Themes</h2>
        <p class="text-xs text-muted-foreground mt-1">Pick a visual theme. Changes apply instantly and persist.</p>
      </div>
      <SettingsSelect
        label="Theme"
        description="Choose one of the built-in themes"
        bind:selected={SETTINGS.accessibility.state["theme"]}
        values={AVAILABLE_THEMES}
      />
    </div>

    <!-- Live Preview Card (avoid primary/accent text as requested) -->
    <div class="rounded-lg border border-border/50 bg-card/30 p-4">
      <div class="text-sm text-muted-foreground">Preview</div>
      <div class="mt-2 grid grid-cols-2 gap-3">
        <div class="rounded-md p-3 bg-background border border-border">
          <div class="text-xs text-muted-foreground">Background</div>
          <div class="text-sm text-foreground">Text on background</div>
        </div>
        <div class="rounded-md p-3 bg-card border border-border">
          <div class="text-xs text-muted-foreground">Card</div>
          <div class="text-sm text-card-foreground">Card foreground</div>
        </div>
        <div class="rounded-md p-3 bg-muted/40 border border-border/60">
          <div class="text-xs text-muted-foreground">Muted Surface</div>
          <div class="text-sm text-foreground">Muted foreground</div>
        </div>
        <div class="rounded-md p-3 bg-popover/40 border border-border/60">
          <div class="text-xs text-muted-foreground">Popover Surface</div>
          <div class="text-sm text-foreground">Popover foreground</div>
        </div>
      </div>
    </div>
  </div>
</Tabs.Content>
