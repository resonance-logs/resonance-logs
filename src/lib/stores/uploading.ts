/**
 * @file Persistent stores for the Uploading feature (API key + progress).
 * Uses @tauri-store/svelte RuneStore for simple, typed persistence.
 */
import { RuneStore } from "@tauri-store/svelte";
import { SETTINGS } from "$lib/settings-store";

export type UploadStatus = "idle" | "in-progress" | "done" | "error";

export interface UploadProgressState extends Record<string, unknown> {
  uploaded: number;
  total: number;
  status: UploadStatus;
  inFlightBatch?: number | null;
  lastError?: string | null;
}

const RUNE_STORE_OPTIONS = { autoStart: true, saveOnChange: true } as const;

const LEGACY_API_KEY_STORE = new RuneStore<{ value: string }>(
  "uploadApiKey",
  { value: "" },
  { autoStart: true, saveOnChange: false }
);

let legacyMigrated = false;
function ensureLegacyApiKeyMigrated() {
  if (legacyMigrated) return;
  legacyMigrated = true;

  const legacyKey = (LEGACY_API_KEY_STORE.state.value || "").trim();
  const currentKey = (SETTINGS.moduleSync.state.apiKey || "").trim();

  if (legacyKey && !currentKey) {
    SETTINGS.moduleSync.state.apiKey = legacyKey;
  }
}

export const UPLOADING = {
  progress: new RuneStore<UploadProgressState>(
    "uploadProgress",
    { uploaded: 0, total: 0, status: "idle" },
    { autoStart: true, saveOnChange: false }
  ),
};

export function setApiKey(key: string) {
  ensureLegacyApiKeyMigrated();
  SETTINGS.moduleSync.state.apiKey = key.trim();
}

export function getApiKey(): string {
  ensureLegacyApiKeyMigrated();
  return (SETTINGS.moduleSync.state.apiKey || "").trim();
}

export function getModuleApiBaseUrl(): string | null {
  const base = (SETTINGS.moduleSync.state.baseUrl || "").trim();
  return base || null;
}

export function resetProgress() {
  UPLOADING.progress.state.uploaded = 0;
  UPLOADING.progress.state.total = 0;
  UPLOADING.progress.state.status = "idle";
  UPLOADING.progress.state.inFlightBatch = null;
  UPLOADING.progress.state.lastError = null;
}

export function setUploading(total: number) {
  UPLOADING.progress.state.total = Math.max(0, total);
  UPLOADING.progress.state.uploaded = 0;
  UPLOADING.progress.state.status = total > 0 ? "in-progress" : "idle";
  UPLOADING.progress.state.inFlightBatch = null;
  UPLOADING.progress.state.lastError = null;
}

export function setProgress(uploaded: number, total?: number) {
  if (typeof total === "number") UPLOADING.progress.state.total = Math.max(0, total);
  UPLOADING.progress.state.uploaded = Math.max(0, uploaded);
  UPLOADING.progress.state.status =
    UPLOADING.progress.state.uploaded >= UPLOADING.progress.state.total && UPLOADING.progress.state.total > 0
      ? "done"
      : "in-progress";
}

export function setError(message: string) {
  UPLOADING.progress.state.lastError = message;
  UPLOADING.progress.state.status = "error";
}
