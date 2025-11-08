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
    resetProgress,
    setUploading,
    setProgress,
    setError,
  } from "$lib/stores/uploading";

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
      await invoke("start_upload", { apiKey: key });
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

<div class="space-y-6">
  <h2 class="text-2xl font-semibold tracking-tight flex items-center gap-2">
    <UploadIcon class="h-6 w-6" /> Uploading
  </h2>
  <p class="text-sm text-muted-foreground">
    Paste your website API key here. Your key is stored locally on your device.
  </p>

  <section class="rounded-lg border p-4 bg-card text-card-foreground space-y-4">
    <div class="flex flex-col gap-2">
      <label for="apiKey" class="text-sm font-medium">API Key</label>
      <div class="flex gap-2 items-center">
        <input
          id="apiKey"
          class="flex-1 min-w-0 rounded-md border bg-background px-3 py-2 outline-none focus:ring-2 focus:ring-primary"
          type={type}
          placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
          value={UPLOADING.apiKey.state.value}
          oninput={(e) => setApiKey((e.target as HTMLInputElement).value)}
          autocomplete="off"
          spellcheck={false}
        />
        <button
          class="inline-flex items-center gap-1 rounded-md border px-3 py-2 hover:bg-accent"
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
          class="inline-flex items-center gap-1 rounded-md border px-3 py-2 hover:bg-accent"
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
        class="inline-flex items-center gap-2 rounded-md bg-primary text-primary-foreground px-4 py-2 disabled:opacity-50"
        onclick={startUpload}
        disabled={busy || !getApiKey()}
      >
        <UploadIcon class="h-4 w-4" />
        <span>Start upload</span>
      </button>
      <button
        class="inline-flex items-center gap-2 rounded-md border px-4 py-2 disabled:opacity-50"
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

  <section class="rounded-lg border p-4 bg-card text-card-foreground space-y-3">
    <h3 class="text-sm font-medium">Progress</h3>
    <div class="flex items-center justify-between text-sm">
      <div class="flex items-center gap-3">
        <span class="font-mono">{UPLOADING.progress.state.uploaded} / {UPLOADING.progress.state.total}</span>
        {#if UPLOADING.progress.state.inFlightBatch != null}
          <span class="text-muted-foreground">Batch {UPLOADING.progress.state.inFlightBatch}</span>
        {/if}
      </div>
      <span class="uppercase tracking-wide text-xs text-muted-foreground">{UPLOADING.progress.state.status}</span>
    </div>
    <div class="h-2 w-full rounded bg-secondary/40 overflow-hidden">
      <div class="h-2 bg-primary transition-all" style={`width: ${pct}%`}></div>
    </div>
    {#if UPLOADING.progress.state.lastError}
      <p class="text-sm text-destructive">{UPLOADING.progress.state.lastError}</p>
    {/if}
  </section>
</div>

<!-- no second script block -->
