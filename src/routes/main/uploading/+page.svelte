<script lang="ts">
  /**
   * Uploading tab UI (Stage 3):
   * - API key input with mask/unmask and persistence
   * - Start/Cancel buttons wired to future Tauri commands
   * - Progress display bound to a persistent store
   * Note: Actual backend upload implementation is Stage 4.
   */
  import { onMount } from "svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { invoke } from "@tauri-apps/api/core";

  import UploadIcon from "virtual:icons/lucide/upload-cloud";
  import EyeIcon from "virtual:icons/lucide/eye";
  import EyeOffIcon from "virtual:icons/lucide/eye-off";
  import RotateCcwIcon from "virtual:icons/lucide/rotate-ccw";
  import XIcon from "virtual:icons/lucide/x";
  import ChevronDown from "virtual:icons/lucide/chevron-down";
  import UploadArwIcon from "virtual:icons/lucide/upload";
  import CheckCircle from "virtual:icons/lucide/check-circle";
  import XCircle from "virtual:icons/lucide/x-circle";
  import AlertTriangle from "virtual:icons/lucide/alert-triangle";
  import RefreshCw from "virtual:icons/lucide/refresh-cw";

  import {
    UPLOADING,
    setApiKey,
    getApiKey,
    getModuleApiBaseUrl,
    resetProgress,
    setUploading,
    setProgress,
    setError,
  } from "$lib/stores/uploading";
  import { SETTINGS } from "$lib/settings-store";
  import SettingsSwitch from "../settings/settings-switch.svelte";
  import SettingsInput from "../settings/settings-input.svelte";
  import { Button } from "$lib/components/ui/button";
  import { commands } from "$lib/bindings";

  let showKey = $state(false);
  let busy = $state(false);
  let infoMsg = $state<string | null>(null);
  // derived attributes to avoid reactivity glitches
  let type = $derived.by<"text" | "password">(() => (showKey ? "text" : "password"));
  let pct = $derived.by<number>(() => {
    const total = UPLOADING.progress.state.total;
    const uploaded = UPLOADING.progress.state.uploaded;
    const ratio = total > 0 ? uploaded / total : 0;
    return Math.min(100, Math.floor(ratio * 100));
  });

  function toggleMask() {
    showKey = !showKey;
  }

  async function startUpload() {
    const key = getApiKey();
    infoMsg = null;
    if (!key) {
      setError("Please enter your API key first.");
      return;
    }
    busy = true;
    resetProgress();
    try {
      // Will be implemented in Stage 4 on the Rust side
      const baseUrl = getModuleApiBaseUrl();
      await invoke("start_upload", { apiKey: key, baseUrl });
      infoMsg = "Starting upload…";
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(`Uploader not available yet: ${msg}`);
      busy = false;
    }
  }

  async function cancelUpload() {
    try {
      await invoke("cancel_upload_cmd");
    } catch {
      // ignore — command will exist in Stage 4
    } finally {
      busy = false;
    }
  }

  function clearKey() {
    setApiKey("");
  }

  // Module sync UI state and helpers (moved from module-sync.svelte)
  let syncStatus = $state<{
    enabled: boolean;
    has_api_key: boolean;
    last_module_count: number;
    base_url: string;
    auto_sync_interval_minutes: number;
    failed_uploads_count: number;
    unknown_attributes_count: number;
  } | null>(null);
  let isSyncing = $state(false);
  let syncResult = $state<{ success: boolean; message: string } | null>(null);
  let mounted = $state(false);
  let unknownAttributes = $state<Array<{ part_id: number; first_seen: string; occurrence_count: number; module_config_ids: number[] }>>([]);
  let showUnknownAttributes = $state(false);
  let expandedSections = $state({ general: true, advanced: false, telemetry: false });

  function toggleSection(section: keyof typeof expandedSections) {
    expandedSections[section] = !expandedSections[section];
  }

  // Load sync status on mount
  onMount(async () => {
    mounted = true;
    await refreshStatus();
  });

  async function refreshStatus() {
    try {
      const result = await commands.getModuleSyncStatus();
      if (result.status === "ok") {
        syncStatus = result.data;
      }
    } catch (e) {
      console.error("Failed to get module sync status:", e);
    }
  }

  // Refresh module sync status when settings change
  $effect(() => {
    if (!mounted) return;
    SETTINGS.moduleSync.state.enabled;
    SETTINGS.moduleSync.state.apiKey;
    SETTINGS.moduleSync.state.baseUrl;
    SETTINGS.moduleSync.state.autoSyncIntervalMinutes;
    refreshStatus().catch((e) => console.error("Failed to refresh module sync status:", e));
  });

  async function triggerSync() {
    isSyncing = true;
    syncResult = null;

    try {
      const result = await commands.triggerModuleSync();
      if (result.status === "ok") {
        const data = result.data;
        syncResult = {
          success: true,
          message: `Successfully synced! Added: ${data.summary.added}, Updated: ${data.summary.updated}, Errors: ${data.summary.errors}`,
        };
        await refreshStatus();
      } else {
        syncResult = { success: false, message: `Sync failed: ${result.error}` };
      }
    } catch (e: any) {
      syncResult = { success: false, message: `Sync failed: ${e.message || e}` };
    } finally {
      isSyncing = false;
      setTimeout(() => {
        syncResult = null;
      }, 5000);
    }
  }

  async function retryFailedUploads() {
    try {
      const result = await commands.retryFailedUploads();
      if (result.status === "ok") {
        syncResult = { success: true, message: `Retried ${result.data.length} failed uploads` };
        await refreshStatus();
      }
    } catch (e: any) {
      syncResult = { success: false, message: `Retry failed: ${e.message || e}` };
    }
  }

  async function loadUnknownAttributes() {
    try {
      const result = await commands.getUnknownAttributes();
      if (result.status === "ok") {
        unknownAttributes = result.data;
        showUnknownAttributes = true;
      }
    } catch (e) {
      console.error("Failed to load unknown attributes:", e);
    }
  }

  async function clearUnknownAttributes() {
    try {
      await commands.clearUnknownAttributes();
      unknownAttributes = [];
      await refreshStatus();
    } catch (e) {
      console.error("Failed to clear unknown attributes:", e);
    }
  }

  onMount(() => {
    const app = getCurrentWebviewWindow();
    const unsubs: Array<() => void> = [];

    app
      .listen<{ total?: number }>("upload:started", (e) => {
        setUploading(Math.max(0, e.payload?.total ?? 0));
        busy = true;
        infoMsg = "Uploading encounters…";
      })
      .then((un) => unsubs.push(un));

    app
      .listen<{ uploaded: number; total?: number; batch?: number }>(
        "upload:progress",
        (e) => {
          setProgress(e.payload.uploaded, e.payload.total);
          UPLOADING.progress.state.inFlightBatch = e.payload.batch ?? null;
        }
      )
      .then((un) => unsubs.push(un));

    app
      .listen("upload:completed", () => {
        busy = false;
        infoMsg = "Upload completed.";
      })
      .then((un) => unsubs.push(un));

    app
      .listen<{ message?: string }>("upload:error", (e) => {
        busy = false;
        setError(e.payload?.message ?? "Unknown upload error");
      })
      .then((un) => unsubs.push(un));

    return () => {
      for (const un of unsubs) un();
    };
  });
</script>

<div class="space-y-4">
  <div class="rounded-lg border border-border/60 bg-card/40 p-6 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]">
    <h2 class="text-2xl font-semibold tracking-tight flex items-center gap-2 text-foreground mb-2">
      <UploadIcon class="h-6 w-6" /> Uploading
    </h2>
    <p class="text-sm text-muted-foreground">
      Paste your website API key here. Your key is stored locally on your device.
    </p>
  </div>

  <section class="rounded-lg border border-border/60 bg-card/40 p-6 space-y-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]">
    <div class="flex flex-col gap-2">
      <label for="apiKey" class="text-sm font-medium text-foreground">API Key</label>
      <div class="flex gap-2 items-center">
        <input
          id="apiKey"
          class="flex-1 min-w-0 rounded-md border border-border bg-popover px-3 py-2 text-foreground outline-none focus:ring-2 focus:ring-primary placeholder:text-muted-foreground/80"
          type={type}
          placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
          value={SETTINGS.moduleSync.state.apiKey}
          oninput={(e) => setApiKey((e.target as HTMLInputElement).value)}
          autocomplete="off"
          spellcheck={false}
        />
        <button
          class="inline-flex items-center gap-1 rounded-md border border-border bg-popover px-3 py-2 text-muted-foreground hover:bg-muted/40 hover:text-foreground transition-colors"
          title={showKey ? "Hide API key" : "Show API key"}
          onclick={toggleMask}
        >
          {#if showKey}
            <EyeOffIcon class="h-4 w-4" />
            <span>Hide</span>
          {:else}
            <EyeIcon class="h-4 w-4" />
            <span>Show</span>
          {/if}
        </button>
        <button
          class="inline-flex items-center gap-1 rounded-md border border-border bg-popover px-3 py-2 text-muted-foreground hover:bg-muted/40 hover:text-foreground transition-colors"
          title="Clear API key"
          onclick={clearKey}
        >
          <XIcon class="h-4 w-4" />
          <span>Clear</span>
        </button>
      </div>
    </div>

    <div class="flex items-center gap-2 pt-2">
      <button
        class="inline-flex items-center gap-2 rounded-md bg-primary text-primary-foreground px-4 py-2 hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={startUpload}
        disabled={busy || !getApiKey()}
      >
        <UploadIcon class="h-4 w-4" />
        <span>Start upload</span>
      </button>
      <button
        class="inline-flex items-center gap-2 rounded-md border border-border bg-popover text-muted-foreground px-4 py-2 hover:bg-muted/40 hover:text-foreground disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        onclick={cancelUpload}
        disabled={!busy}
      >
        <RotateCcwIcon class="h-4 w-4" />
        <span>Cancel</span>
      </button>
      {#if infoMsg}
        <span class="text-xs text-muted-foreground">{infoMsg}</span>
      {/if}
    </div>
  </section>

  <section class="rounded-lg border border-border/60 bg-card/40 p-6 space-y-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]">
    <h3 class="text-base font-semibold text-foreground">Progress</h3>
    <div class="flex items-center justify-between text-sm">
      <div class="flex items-center gap-3 text-muted-foreground">
        <span class="font-mono">{UPLOADING.progress.state.uploaded} / {UPLOADING.progress.state.total}</span>
        {#if UPLOADING.progress.state.inFlightBatch != null}
          <span class="text-muted-foreground">Batch {UPLOADING.progress.state.inFlightBatch}</span>
        {/if}
      </div>
      <span class="uppercase tracking-wide text-xs text-muted-foreground">{UPLOADING.progress.state.status}</span>
    </div>
    <div class="h-2 w-full rounded bg-muted/50 overflow-hidden">
      <div class="h-2 bg-primary transition-all" style={`width: ${pct}%`}></div>
    </div>
    {#if UPLOADING.progress.state.lastError}
      <p class="text-sm text-destructive">{UPLOADING.progress.state.lastError}</p>
    {/if}
  </section>

  <!-- Module Sync / Optimizer UI (moved from settings) -->
  <section class="rounded-lg border border-border/60 bg-card/40 p-6 space-y-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]">
    <div class="flex items-center justify-between">
      <h3 class="text-base font-semibold text-foreground">Module Optimizer</h3>
      <div class="text-sm text-muted-foreground">Automatically upload modules to the website</div>
    </div>

    <div class="space-y-3">
      <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
        <button
          type="button"
          class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
          onclick={() => toggleSection('general')}
        >
          <h2 class="text-base font-semibold text-foreground">Module Sync Configuration</h2>
          <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.general ? 'rotate-180' : ''}" />
        </button>
        {#if expandedSections.general}
          <div class="px-4 pb-3 space-y-1">
            <SettingsSwitch
              bind:checked={SETTINGS.moduleSync.state.enabled}
              label="Enable Module Sync"
              description="Automatically upload module data to the resonance-website optimizer when you open your inventory in-game"
            />

            {#if syncStatus}
              <div class="py-2.5 px-3 rounded-md bg-popover/30">
                <div class="text-xs text-muted-foreground space-y-1">
                  <div class="flex items-center justify-between">
                    <span>Status:</span>
                    <span class="font-medium {syncStatus.enabled ? 'text-green-400' : 'text-muted-foreground'}">
                      {syncStatus.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </div>
                  <div class="flex items-center justify-between">
                    <span>API Key:</span>
                    <span class="font-medium {syncStatus.has_api_key ? 'text-green-400' : 'text-yellow-400'}">
                      {syncStatus.has_api_key ? 'Configured' : 'Not Set'}
                    </span>
                  </div>
                  <div class="flex items-center justify-between">
                    <span>Cached Modules:</span>
                    <span class="font-medium text-foreground">{syncStatus.last_module_count}</span>
                  </div>
                  <div class="flex items-center justify-between">
                    <span>Failed Uploads:</span>
                    <span class="font-medium {syncStatus.failed_uploads_count > 0 ? 'text-yellow-400' : 'text-green-400'}">
                      {syncStatus.failed_uploads_count}
                    </span>
                  </div>
                  {#if syncStatus.unknown_attributes_count > 0}
                    <div class="flex items-center justify-between">
                      <span>Unknown Attributes:</span>
                      <span class="font-medium text-yellow-400">{syncStatus.unknown_attributes_count}</span>
                    </div>
                  {/if}
                </div>
              </div>
            {/if}

            <div class="pt-2">
              <Button
                onclick={triggerSync}
                disabled={isSyncing || !SETTINGS.moduleSync.state.enabled || !SETTINGS.moduleSync.state.apiKey}
                class="w-full"
                variant="default"
              >
                {#if isSyncing}
                  <div class="flex items-center gap-2">
                    <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"></div>
                    Syncing...
                  </div>
                {:else}
                  <div class="flex items-center gap-2">
                    <UploadArwIcon class="w-4 h-4" />
                    Sync Modules Now
                  </div>
                {/if}
              </Button>
            </div>

            {#if syncStatus && syncStatus.failed_uploads_count > 0}
              <div class="pt-2">
                <Button
                  onclick={retryFailedUploads}
                  class="w-full"
                  variant="outline"
                >
                  <div class="flex items-center gap-2">
                    <RefreshCw class="w-4 h-4" />
                    Retry Failed Uploads ({syncStatus.failed_uploads_count})
                  </div>
                </Button>
              </div>
            {/if}

            {#if syncResult}
              <div class="mt-2 px-3 py-2 rounded-md {syncResult.success ? 'bg-green-500/10 border border-green-500/20' : 'bg-red-500/10 border border-red-500/20'}">
                <div class="flex items-start gap-2 text-sm">
                  {#if syncResult.success}
                    <CheckCircle class="w-4 h-4 text-green-400 mt-0.5 shrink-0" />
                    <span class="text-green-400">{syncResult.message}</span>
                  {:else}
                    <XCircle class="w-4 h-4 text-red-400 mt-0.5 shrink-0" />
                    <span class="text-red-400">{syncResult.message}</span>
                  {/if}
                </div>
              </div>
            {/if}

            <div class="pt-2 px-3 py-2 rounded-md bg-blue-500/10 border border-blue-500/20">
              <div class="text-xs text-blue-300 space-y-1">
                <p class="font-medium">How it works:</p>
                <ul class="list-disc list-inside space-y-0.5 text-blue-200/80">
                  <li>Open your inventory in-game to trigger module data capture</li>
                  <li>Modules are automatically extracted and cached</li>
                  <li>If sync is enabled, modules upload automatically to the website</li>
                  <li>Use "Sync Modules Now" to manually upload cached modules</li>
                </ul>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Advanced -> Interval setting -->
      <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
        <button
          type="button"
          class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
          onclick={() => toggleSection('advanced')}
        >
          <h2 class="text-base font-semibold text-foreground">Advanced Settings</h2>
          <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.advanced ? 'rotate-180' : ''}" />
        </button>
        {#if expandedSections.advanced}
          <div class="px-4 pb-3 space-y-1">
            <SettingsInput
              bind:value={SETTINGS.moduleSync.state.autoSyncIntervalMinutes}
              type="number"
              label="Auto-Sync Interval (minutes)"
              description="Automatically sync modules every N minutes. Set to 0 to disable scheduled sync."
              placeholder="0"
            />
          </div>
        {/if}
      </div>

      {#if syncStatus && syncStatus.unknown_attributes_count > 0}
        <div class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]">
          <button
            type="button"
            class="w-full flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors"
            onclick={() => toggleSection('telemetry')}
          >
            <h2 class="text-base font-semibold text-foreground flex items-center gap-2">
              <AlertTriangle class="w-5 h-5 text-yellow-400" />
              Unknown Attributes Detected ({syncStatus.unknown_attributes_count})
            </h2>
            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedSections.telemetry ? 'rotate-180' : ''}" />
          </button>
          {#if expandedSections.telemetry}
            <div class="px-4 pb-3 space-y-2">
              <div class="px-3 py-2 rounded-md bg-blue-500/10 border border-blue-500/20">
                <div class="text-xs text-blue-300">
                  <p class="font-medium mb-1">ℹ️ What is this?</p>
                  <p class="text-blue-200/80">The app has detected module attributes that are not in our database. This usually means the game has been updated with new module types. These attributes have been temporarily labeled as "未知属性(ID)" in your modules.</p>
                </div>
              </div>

              <div class="flex gap-2">
                <Button
                  onclick={loadUnknownAttributes}
                  class="flex-1"
                  variant="outline"
                >
                  View Unknown Attributes
                </Button>
                <Button
                  onclick={clearUnknownAttributes}
                  class="flex-1"
                  variant="outline"
                >
                  Clear Tracking
                </Button>
              </div>

              {#if showUnknownAttributes && unknownAttributes.length > 0}
                <div class="mt-2 px-3 py-2 rounded-md bg-popover/30 max-h-64 overflow-y-auto">
                  <div class="text-xs text-muted-foreground space-y-2">
                    {#each unknownAttributes as attr}
                      <div class="border-b border-border/30 pb-2">
                        <div class="flex items-center justify-between mb-1">
                          <span class="font-medium text-foreground">Part ID: {attr.part_id}</span>
                          <span class="text-yellow-400">Count: {attr.occurrence_count}</span>
                        </div>
                        <div class="text-xs">
                          <div>First seen: {new Date(attr.first_seen).toLocaleString()}</div>
                          <div>Found in modules: {attr.module_config_ids.join(', ')}</div>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </section>
</div>

<!-- no second script block -->
