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
  import RotateCwIcon from "virtual:icons/lucide/rotate-cw";
  import XIcon from "virtual:icons/lucide/x";
  import CheckIcon from "virtual:icons/lucide/check";

  import {
    UPLOADING,
    setApiKey,
    getApiKey,
    getModuleApiBaseUrl,
    resetProgress,
    setUploading,
    setProgress,
    setError,
    setPlayerDataSyncTime,
    setPlayerDataDetectedTime,
    addApiLog,
    clearApiLogs,
  } from "$lib/stores/uploading";
  import { SETTINGS } from "$lib/settings-store";

  let showKey = $state(false);
  let showGuide = $state(false);
  let showLogsDropdown = $state(false);
  let busy = $state(false);
  let checking = $state(false);
  let checkResult = $state<boolean | null>(null);
  let infoMsg = $state<string | null>(null);
  let apiKeyInput = $state(getApiKey());
  let lastPersistedKey = $state("");

  // derived attributes to avoid reactivity glitches
  let type = $derived.by<"text" | "password">(() =>
    showKey ? "text" : "password",
  );
  let pct = $derived.by<number>(() => {
    const total = UPLOADING.progress.state.total;
    const uploaded = UPLOADING.progress.state.uploaded;
    const ratio = total > 0 ? uploaded / total : 0;
    return Math.min(100, Math.floor(ratio * 100));
  });

  function formatRelativeTime(timestampMs: number): string {
    const now = Date.now();
    const diffMs = now - timestampMs;
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHour = Math.floor(diffMin / 60);
    const diffDays = Math.floor(diffHour / 24);

    if (diffSec < 60) return `${diffSec}s ago`;
    if (diffMin < 60) return `${diffMin}m ago`;
    if (diffHour < 24) return `${diffHour}h ago`;
    return `${diffDays}d ago`;
  }

  function toggleMask() {
    showKey = !showKey;
  }

  function toggleGuide() {
    showGuide = !showGuide;
  }

  function saveApiKey() {
    const trimmed = apiKeyInput.trim();
    setApiKey(trimmed);
    lastPersistedKey = trimmed;
    infoMsg = trimmed
      ? "API key saved."
      : "API key cleared. Uploads are disabled.";
  }

  async function checkKey() {
    const trimmed = apiKeyInput.trim();
    if (!trimmed) {
      infoMsg = "Please enter an API key first.";
      return;
    }
    checking = true;
    checkResult = null;
    infoMsg = null;
    try {
      const baseUrl = getModuleApiBaseUrl();
      const valid = await invoke<boolean>("check_api_key", {
        apiKey: trimmed,
        baseUrl,
      });
      checkResult = valid;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      infoMsg = `Check failed: ${msg}`;
      checkResult = false;
    } finally {
      checking = false;
    }
  }

  async function startUpload() {
    const key = getApiKey();
    const trimmedInput = apiKeyInput.trim();
    infoMsg = null;
    if (!key) {
      setError("Please enter your API key, click Save, and try again.");
      return;
    }
    if (trimmedInput !== key) {
      setError(
        "You have unsaved API key changes. Click Save before starting an upload.",
      );
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

  async function forceRecheck() {
    const key = getApiKey();
    const trimmedInput = apiKeyInput.trim();
    infoMsg = null;
    if (!key) {
      setError("Please enter your API key, click Save, and try again.");
      return;
    }
    if (trimmedInput !== key) {
      setError(
        "You have unsaved API key changes. Click Save before forcing a recheck.",
      );
      return;
    }
    busy = true;
    resetProgress();
    addApiLog("info", "Force recheck requested…");
    try {
      const baseUrl = getModuleApiBaseUrl();
      // Start the upload recheck
      await invoke("start_upload", { apiKey: key, baseUrl });
      infoMsg = "Recheck started…";
      // Also manually request a player-data sync to ensure player build data is updated immediately
      try {
        // Set the recheck message up-front so it doesn't overwrite upload completion if upload finishes
        infoMsg = "Recheck started… checking logs & player data";
        await invoke("sync_player_data", { apiKey: key, baseUrl });
      } catch (err) {
        // Player data sync may not be available yet; don't block recheck, but surface a helpful message
        const msg = err instanceof Error ? err.message : String(err);
        setError(`Player data sync failed to start: ${msg}`);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(`Uploader not available yet: ${msg}`);
      busy = false;
    }
  }

  function clearKey() {
    apiKeyInput = "";
  }

  $effect(() => {
    const stored = (SETTINGS.moduleSync.state.apiKey || "").trim();
    if (stored !== lastPersistedKey) {
      lastPersistedKey = stored;
      apiKeyInput = stored;
    }
  });

  onMount(() => {
    const app = getCurrentWebviewWindow();
    const unsubs: Array<() => void> = [];
    let isDestroyed = false;

    // Fetch player data times on mount
    invoke<{ lastDetectedMs?: number | null; lastSyncMs?: number | null }>(
      "get_player_data_times",
    )
      .then((result) => {
        if (isDestroyed) return;
        if (result.lastDetectedMs) {
          setPlayerDataDetectedTime(result.lastDetectedMs);
        }
        if (result.lastSyncMs) {
          setPlayerDataSyncTime(result.lastSyncMs);
        }
      })
      .catch((err) => {
        console.warn("Failed to fetch player data times:", err);
      });

    // Helper to safely add unsubscribe function only if component isn't destroyed
    const safeAddUnsub = (un: () => void) => {
      if (isDestroyed) {
        // Component was destroyed before listener was set up, clean up immediately
        un();
      } else {
        unsubs.push(un);
      }
    };

    app
      .listen<{ total?: number }>("upload:started", (e) => {
        if (isDestroyed) return;
        setUploading(Math.max(0, e.payload?.total ?? 0));
        busy = true;
        infoMsg = "Uploading encounters…";
        addApiLog(
          "info",
          `Upload started: ${e.payload?.total ?? 0} encounter(s) to process`,
        );
      })
      .then(safeAddUnsub);

    app
      .listen<{
        uploaded: number;
        total?: number;
        succeeded?: number;
        errored?: number;
        skipped?: number;
        message?: string;
      }>("upload:progress", (e) => {
        if (isDestroyed) return;
        setProgress(
          e.payload.uploaded,
          e.payload.total,
          e.payload.succeeded,
          e.payload.errored,
        );
        if (e.payload.message) {
          addApiLog("info", e.payload.message);
        } else if (e.payload.skipped) {
          addApiLog("info", `Skipped ${e.payload.skipped} duplicate(s)`);
        }
      })
      .then(safeAddUnsub);

    app
      .listen<{
        uploaded?: number;
        total?: number;
        succeeded?: number;
        errored?: number;
      }>("upload:completed", (e) => {
        if (isDestroyed) return;
        busy = false;
        infoMsg = "Upload completed.";
        const succ = e.payload?.succeeded ?? e.payload?.uploaded ?? 0;
        const err = e.payload?.errored ?? 0;
        addApiLog(
          "success",
          `Upload completed: ${succ} synced, ${err} errored`,
        );
      })
      .then(safeAddUnsub);

    app
      .listen<{ message?: string }>("upload:error", (e) => {
        if (isDestroyed) return;
        busy = false;
        const msg = e.payload?.message ?? "Unknown upload error";
        setError(msg);
        addApiLog("error", `Upload error: ${msg}`);
      })
      .then(safeAddUnsub);

    // Player data sync events
    app
      .listen<{ total?: number }>("player-data-sync:started", (e) => {
        if (isDestroyed) return;
        addApiLog(
          "info",
          `Player data sync started: ${e.payload?.total ?? 0} potential entries`,
        );
      })
      .then(safeAddUnsub);

    app
      .listen<{ synced?: number; total?: number }>(
        "player-data-sync:completed",
        (e) => {
          if (isDestroyed) return;
          setPlayerDataSyncTime(Date.now());
          addApiLog(
            "success",
            `Player data synced: ${e.payload?.synced ?? 0} entries`,
          );
        },
      )
      .then(safeAddUnsub);

    app
      .listen<{ message?: string }>("player-data-sync:error", (e) => {
        if (isDestroyed) return;
        addApiLog(
          "error",
          `Player data sync error: ${e.payload?.message ?? "Unknown"}`,
        );
      })
      .then(safeAddUnsub);

    return () => {
      isDestroyed = true;
      for (const un of unsubs) un();
    };
  });
</script>

<div class="space-y-4">
  <div
    class="rounded-lg border border-border/60 bg-card/40 p-6 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]"
  >
    <h2
      class="text-2xl font-semibold tracking-tight flex items-center gap-2 text-foreground mb-2"
    >
      <UploadIcon class="h-6 w-6" /> Uploading
    </h2>
    <p class="text-sm text-muted-foreground">
      Paste your website API key here and click <strong>Save</strong>. No logs
      leave your PC until a saved key exists, and removing the key + saving
      disables uploads again.
    </p>
  </div>

  <section
    class="rounded-lg border border-border/60 bg-card/40 p-6 space-y-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]"
  >
    <div class="flex items-start justify-between gap-4">
      <div>
        <h3 class="text-base font-semibold text-foreground">
          Uploader user guide
        </h3>
        <p class="text-sm text-muted-foreground">
          How to upload your logs (OPTIONAL)
        </p>
      </div>
      <button
        class="inline-flex shrink-0 items-center gap-2 rounded-md border border-border bg-popover px-3 py-2 text-xs uppercase tracking-wide text-muted-foreground hover:bg-muted/40 hover:text-foreground transition-colors"
        onclick={toggleGuide}
      >
        {#if showGuide}
          Hide guide
        {:else}
          Show guide
        {/if}
      </button>
    </div>
    {#if showGuide}
      <div
        class="rounded-md border border-border/40 bg-background/60 p-4 space-y-2 text-sm text-muted-foreground"
      >
        <p class="text-xs uppercase tracking-wide text-muted-foreground/80">
          How uploads work
        </p>
        <ol class="list-decimal pl-4 space-y-2">
          <li>
            <strong>Grab your API key</strong> from
            <code class="text-foreground break-all"
              >bpsr.app &rarr; Profile &rarr; API</code
            >.
          </li>
          <li>
            <strong>Paste it below and click Save.</strong> Nothing is uploaded until
            a saved API key exists, including auto-upload/background tasks.
          </li>
          <li>
            <strong>Click Start upload</strong> after you add your API key click
            start upload to upload any existing logs.
          </li>
          <li>
            <strong>Uploads happen in the background:</strong> after adding your
            API key your logs are autosync'd with the website.
          </li>
          <li>
            <strong>Opt out any time:</strong> clear the API key field, click Save,
            and the uploader immediately stops sending data.
          </li>
        </ol>
      </div>
    {/if}
  </section>

  <section
    class="rounded-lg border border-border/60 bg-card/40 p-6 space-y-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]"
  >
    <div class="flex flex-col gap-2">
      <label for="apiKey" class="text-sm font-medium text-foreground"
        >API Key</label
      >
      <div class="flex gap-2 items-center">
        <input
          id="apiKey"
          class="flex-1 min-w-0 rounded-md border border-border bg-popover px-3 py-2 text-foreground outline-none focus:ring-2 focus:ring-primary placeholder:text-muted-foreground/80"
          {type}
          placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
          value={apiKeyInput}
          oninput={(e) => {
            apiKeyInput = (e.target as HTMLInputElement).value;
            checkResult = null;
          }}
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
        <button
          class="inline-flex items-center gap-1 rounded-md bg-primary text-primary-foreground px-3 py-2 hover:opacity-90 transition-colors"
          title="Save API key"
          onclick={saveApiKey}
        >
          <span>Save</span>
        </button>
        <button
          class="inline-flex items-center gap-1 rounded-md px-3 py-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors {checkResult ===
          true
            ? 'bg-green-600 text-white hover:bg-green-700'
            : checkResult === false
              ? 'bg-red-600 text-white hover:bg-red-700'
              : 'border border-border bg-popover text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
          title="Check if API key is valid"
          onclick={checkKey}
          disabled={checking || !apiKeyInput.trim()}
        >
          <CheckIcon class="h-4 w-4" />
          <span
            >{checking
              ? "Checking…"
              : checkResult === true
                ? "Valid"
                : checkResult === false
                  ? "Invalid"
                  : "Check"}</span
          >
        </button>
      </div>
      <div class="flex items-center gap-2 pt-1">
        <label
          class="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground cursor-pointer select-none transition-colors"
        >
          <input
            type="checkbox"
            class="h-4 w-4 rounded border-border bg-popover text-primary focus:ring-primary"
            checked={SETTINGS.moduleSync.state.autoUpload ?? true}
            onchange={(e) =>
              (SETTINGS.moduleSync.state.autoUpload = e.currentTarget.checked)}
          />
          <span>Auto-upload logs</span>
        </label>
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
        onclick={forceRecheck}
        disabled={busy || !getApiKey()}
        title="Force recheck all logs and player data with server and upload if valid"
      >
        <RotateCwIcon class="h-4 w-4" />
        <span>Force Recheck</span>
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

  <section
    class="rounded-lg border border-border/60 bg-card/40 p-6 space-y-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]"
  >
    <div class="flex items-center justify-between">
      <h3 class="text-base font-semibold text-foreground">Progress</h3>
      <div class="relative">
        <button
          class="inline-flex items-center gap-1 rounded-md border border-border bg-popover px-2 py-1 text-xs text-muted-foreground hover:bg-muted/40 hover:text-foreground transition-colors"
          onclick={() => (showLogsDropdown = !showLogsDropdown)}
          title="View API request logs"
        >
          <span>View Logs</span>
          <span class="text-[10px]">{showLogsDropdown ? "▲" : "▼"}</span>
        </button>
        {#if showLogsDropdown && (UPLOADING.progress.state.apiLogs?.length ?? 0) > 0}
          <button
            class="absolute right-0 top-full mt-1 text-[10px] text-muted-foreground hover:text-foreground"
            onclick={() => clearApiLogs()}
          >
            Clear
          </button>
        {/if}
      </div>
    </div>
    {#if showLogsDropdown}
      <div
        class="max-h-40 overflow-y-auto rounded border border-border/40 bg-background/60 p-2 space-y-1 text-xs"
      >
        {#if (UPLOADING.progress.state.apiLogs?.length ?? 0) === 0}
          <p class="text-muted-foreground italic">No API logs yet.</p>
        {:else}
          {#each UPLOADING.progress.state.apiLogs as log (log.id)}
            <div class="flex gap-2 items-start">
              <span
                class="text-[10px] text-muted-foreground/60 shrink-0 font-mono"
              >
                {new Date(log.timestamp).toLocaleTimeString()}
              </span>
              <span
                class={log.type === "error"
                  ? "text-red-400"
                  : log.type === "success"
                    ? "text-green-400"
                    : log.type === "warning"
                      ? "text-yellow-400"
                      : "text-muted-foreground"}
              >
                {log.message}
              </span>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
    <div class="flex items-center justify-between text-sm">
      <div class="flex items-center gap-3 text-muted-foreground">
        <span class="font-mono"
          >{UPLOADING.progress.state.uploaded} / {UPLOADING.progress.state
            .total}</span
        >
        {#if UPLOADING.progress.state.succeeded > 0 || UPLOADING.progress.state.errored > 0}
          <span class="text-xs">
            (<span class="text-green-500"
              >{UPLOADING.progress.state.succeeded} synced</span
            >{#if UPLOADING.progress.state.errored > 0}, <span
                class="text-red-500"
                >{UPLOADING.progress.state.errored} errored</span
              >{/if})
          </span>
        {/if}
      </div>
      <span class="uppercase tracking-wide text-xs text-muted-foreground"
        >{UPLOADING.progress.state.status}</span
      >
    </div>
    <div class="relative h-4 w-full rounded bg-muted/50 overflow-hidden">
      <div
        class="h-4 bg-primary transition-all flex items-center justify-center"
        style={`width: ${pct}%`}
      >
        {#if pct > 10}
          <span class="text-[10px] font-medium text-primary-foreground"
            >{pct}%</span
          >
        {/if}
      </div>
      {#if pct <= 10 && UPLOADING.progress.state.total > 0}
        <span
          class="absolute inset-0 flex items-center justify-center text-[10px] font-medium text-muted-foreground"
          >{pct}%</span
        >
      {/if}
    </div>
    {#if UPLOADING.progress.state.lastError}
      <p class="text-sm text-destructive">
        {UPLOADING.progress.state.lastError}
      </p>
    {/if}
  </section>

  <section
    class="rounded-lg border border-border/60 bg-card/40 p-6 space-y-3 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]"
  >
    <h3 class="text-base font-semibold text-foreground">Player Data</h3>
    <div class="flex flex-col gap-2 text-sm text-muted-foreground">
      <div class="flex items-center gap-2">
        <span>Synced:</span>
        <span class="font-mono">
          {#if UPLOADING.progress.state.playerDataLastSyncMs}
            {formatRelativeTime(UPLOADING.progress.state.playerDataLastSyncMs)}
          {:else}
            Never
          {/if}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <span>Last detected:</span>
        <span class="font-mono">
          {#if UPLOADING.progress.state.playerDataLastDetectedMs}
            {formatRelativeTime(
              UPLOADING.progress.state.playerDataLastDetectedMs,
            )}
          {:else}
            N/A
          {/if}
        </span>
      </div>
    </div>
  </section>
</div>

<!-- no second script block -->
