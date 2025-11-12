<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import SettingsSwitch from "./settings-switch.svelte";
  import SettingsInput from "./settings-input.svelte";
  import { Button } from "$lib/components/ui/button";
  import { SETTINGS } from "$lib/settings-store";
  import { commands } from "$lib/bindings";
  import { onMount } from 'svelte';
  import ChevronDown from "virtual:icons/lucide/chevron-down";
  import Upload from "virtual:icons/lucide/upload";
  import CheckCircle from "virtual:icons/lucide/check-circle";
  import XCircle from "virtual:icons/lucide/x-circle";
  import AlertTriangle from "virtual:icons/lucide/alert-triangle";
  import RefreshCw from "virtual:icons/lucide/refresh-cw";

  const SETTINGS_CATEGORY = "moduleSync";

  // Sync state
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

  // Collapsible section state
  let expandedSections = $state({
    general: true,
    advanced: false,
    telemetry: false,
  });

  function toggleSection(section: keyof typeof expandedSections) {
    expandedSections[section] = !expandedSections[section];
  }

  // Load status on mount
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

  // Sync settings to backend when they change
  $effect(() => {
    if (mounted) {
      const enabled = SETTINGS.moduleSync.state.enabled;
      const apiKey = SETTINGS.moduleSync.state.apiKey;
      const baseUrl = SETTINGS.moduleSync.state.baseUrl;
      const autoSyncInterval = SETTINGS.moduleSync.state.autoSyncIntervalMinutes || 0;

      // Update backend config
      commands.setModuleSyncConfig(enabled, apiKey || null, baseUrl || null, autoSyncInterval)
        .then(() => refreshStatus())
        .catch(e => console.error("Failed to update module sync config:", e));
    }
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
          message: `Successfully synced! Added: ${data.summary.added}, Updated: ${data.summary.updated}, Errors: ${data.summary.errors}`
        };
        await refreshStatus();
      } else {
        syncResult = {
          success: false,
          message: `Sync failed: ${result.error}`
        };
      }
    } catch (e: any) {
      syncResult = {
        success: false,
        message: `Sync failed: ${e.message || e}`
      };
    } finally {
      isSyncing = false;
      // Clear result after 5 seconds
      setTimeout(() => {
        syncResult = null;
      }, 5000);
    }
  }

  async function retryFailedUploads() {
    try {
      const result = await commands.retryFailedUploads();
      if (result.status === "ok") {
        syncResult = {
          success: true,
          message: `Retried ${result.data.length} failed uploads`
        };
        await refreshStatus();
      }
    } catch (e: any) {
      syncResult = {
        success: false,
        message: `Retry failed: ${e.message || e}`
      };
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
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <div class="space-y-3">
    <!-- General Settings -->
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

          <SettingsInput
            bind:value={SETTINGS.moduleSync.state.apiKey}
            type="password"
            label="API Key"
            description="Your API key from resonance-website. Get it from your account settings."
            placeholder="Enter your API key"
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
                  <Upload class="w-4 h-4" />
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

    <!-- Advanced Settings -->
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
            bind:value={SETTINGS.moduleSync.state.baseUrl}
            type="url"
            label="API Base URL"
            description="The base URL for the resonance-website API. Only change if you're using a custom deployment."
            placeholder="http://localhost:8080/api/v1"
          />

          <SettingsInput
            bind:value={SETTINGS.moduleSync.state.autoSyncIntervalMinutes}
            type="number"
            label="Auto-Sync Interval (minutes)"
            description="Automatically sync modules every N minutes. Set to 0 to disable scheduled sync."
            placeholder="0"
          />

          <div class="px-3 py-2 rounded-md bg-yellow-500/10 border border-yellow-500/20">
            <div class="text-xs text-yellow-300">
              <p class="font-medium mb-1">⚠️ Warning:</p>
              <p class="text-yellow-200/80">Only modify the API Base URL if you know what you're doing. Incorrect values will prevent module sync from working.</p>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Telemetry Section -->
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
</Tabs.Content>
