/**
 * @file Persistent stores for the Uploading feature (API key + progress).
 * Uses @tauri-store/svelte RuneStore for simple, typed persistence.
 */
import { RuneStore } from "@tauri-store/svelte";

export type UploadStatus = "idle" | "in-progress" | "done" | "error";

export interface UploadProgressState extends Record<string, unknown> {
  uploaded: number;
  total: number;
  status: UploadStatus;
  inFlightBatch?: number | null;
  lastError?: string | null;
}

const RUNE_STORE_OPTIONS = { autoStart: true, saveOnChange: true } as const;

export const UPLOADING = {
  apiKey: new RuneStore<{ value: string }>(
    "uploadApiKey",
    { value: "" },
    RUNE_STORE_OPTIONS
  ),
  progress: new RuneStore<UploadProgressState>(
    "uploadProgress",
    { uploaded: 0, total: 0, status: "idle" },
    { autoStart: true, saveOnChange: false }
  ),
};

export function setApiKey(key: string) {
  UPLOADING.apiKey.state.value = key.trim();
}

export function getApiKey(): string {
  return (UPLOADING.apiKey.state.value || "").trim();
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
