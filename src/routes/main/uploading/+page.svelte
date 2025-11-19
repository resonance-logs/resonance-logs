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

  let showKey = $state(false);
  let showGuide = $state(false);
  let busy = $state(false);
  let infoMsg = $state<string | null>(null);
  let apiKeyInput = $state(getApiKey());
  let lastPersistedKey = apiKeyInput;
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

  function toggleGuide() {
    showGuide = !showGuide;
  }

  function saveApiKey() {
    const trimmed = apiKeyInput.trim();
    setApiKey(trimmed);
    lastPersistedKey = trimmed;
    infoMsg = trimmed ? "API key saved." : "API key cleared. Uploads are disabled.";
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
      setError("You have unsaved API key changes. Click Save before starting an upload.");
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
      Paste your website API key here and click <strong>Save</strong>. No logs leave your PC until a saved key exists, and removing the key + saving disables uploads again.
    </p>
  </div>

  <section class="rounded-lg border border-border/60 bg-card/40 p-6 space-y-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]">
    <div class="flex items-start justify-between gap-4">
      <div>
        <h3 class="text-base font-semibold text-foreground">Uploader user guide</h3>
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
      <div class="rounded-md border border-border/40 bg-background/60 p-4 space-y-2 text-sm text-muted-foreground">
        <p class="text-xs uppercase tracking-wide text-muted-foreground/80">How uploads work</p>
        <ol class="list-decimal pl-4 space-y-2">
          <li>
            <strong>Grab your API key</strong> from <code class="text-foreground break-all">bpsr.app &rarr; Profile &rarr; API</code>.
          </li>
          <li>
            <strong>Paste it below and click Save.</strong> Nothing is uploaded until a saved API key exists, including auto-upload/background tasks.
          </li>
          <li>
            <strong>Click Start upload</strong> after you add your API key click start upload to upload any existing logs.
          </li>
          <li>
          <strong>Uploads happen in the background:</strong> after adding your API key your logs are autosync'd with the website.
          </li>
          <li>
            <strong>Opt out any time:</strong> clear the API key field, click Save, and the uploader immediately stops sending data.
          </li>
        </ol>
      </div>
    {/if}
  </section>

  <section class="rounded-lg border border-border/60 bg-card/40 p-6 space-y-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]">
    <div class="flex flex-col gap-2">
      <label for="apiKey" class="text-sm font-medium text-foreground">API Key</label>
      <div class="flex gap-2 items-center">
        <input
          id="apiKey"
          class="flex-1 min-w-0 rounded-md border border-border bg-popover px-3 py-2 text-foreground outline-none focus:ring-2 focus:ring-primary placeholder:text-muted-foreground/80"
          type={type}
          placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
          value={apiKeyInput}
          oninput={(e) => (apiKeyInput = (e.target as HTMLInputElement).value)}
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

</div>

<!-- no second script block -->
