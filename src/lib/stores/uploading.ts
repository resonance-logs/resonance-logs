/**
 * @file Persistent stores for the Uploading feature (API key + progress).
 * Uses @tauri-store/svelte RuneStore for simple, typed persistence.
 */
import { RuneStore } from "@tauri-store/svelte";
import { SETTINGS } from "$lib/settings-store";

export type UploadStatus = "idle" | "in-progress" | "done" | "error";

export type ApiLogType = "info" | "success" | "error" | "warning";

export interface ApiLogEntry {
  id: number;
  timestamp: number;
  type: ApiLogType;
  message: string;
}

export interface UploadProgressState extends Record<string, unknown> {
  uploaded: number;
  total: number;
  succeeded: number;
  errored: number;
  status: UploadStatus;
  lastError?: string | null;
  /** Timestamp (ms since epoch) when player data was last synced to server */
  playerDataLastSyncMs?: number | null;
  /** Timestamp (ms since epoch) when player data was last detected locally */
  playerDataLastDetectedMs?: number | null;
  /** Recent API request/response logs */
  apiLogs: ApiLogEntry[];
}

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
    { uploaded: 0, total: 0, succeeded: 0, errored: 0, status: "idle", apiLogs: [] },
    { autoStart: true, saveOnChange: false }
  ),
};


const MAX_API_LOGS = 50;

export function addApiLog(type: ApiLogType, message: string) {
  const logs = UPLOADING.progress.state.apiLogs ?? [];
  const maxId = logs.reduce((max, log) => Math.max(max, log.id), 0);
  const entry: ApiLogEntry = {
    id: maxId + 1,
    timestamp: Date.now(),
    type,
    message,
  };
  // Add to front, limit to MAX_API_LOGS
  UPLOADING.progress.state.apiLogs = [entry, ...logs].slice(0, MAX_API_LOGS);
}

export function clearApiLogs() {
  UPLOADING.progress.state.apiLogs = [];
}

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
  UPLOADING.progress.state.succeeded = 0;
  UPLOADING.progress.state.errored = 0;
  UPLOADING.progress.state.status = "idle";
  UPLOADING.progress.state.lastError = null;
}

export function setUploading(total: number) {
  UPLOADING.progress.state.total = Math.max(0, total);
  UPLOADING.progress.state.uploaded = 0;
  UPLOADING.progress.state.succeeded = 0;
  UPLOADING.progress.state.errored = 0;
  UPLOADING.progress.state.status = total > 0 ? "in-progress" : "idle";
  UPLOADING.progress.state.lastError = null;
}

export function setProgress(uploaded: number, total?: number, succeeded?: number, errored?: number) {
  if (typeof total === "number") UPLOADING.progress.state.total = Math.max(0, total);
  UPLOADING.progress.state.uploaded = Math.max(0, uploaded);
  if (typeof succeeded === "number") UPLOADING.progress.state.succeeded = succeeded;
  if (typeof errored === "number") UPLOADING.progress.state.errored = errored;
  UPLOADING.progress.state.status =
    UPLOADING.progress.state.uploaded >= UPLOADING.progress.state.total && UPLOADING.progress.state.total > 0
      ? "done"
      : "in-progress";
}

export function setError(message: string) {
  UPLOADING.progress.state.lastError = message;
  UPLOADING.progress.state.status = "error";
}

export function setPlayerDataSyncTime(syncMs: number) {
  UPLOADING.progress.state.playerDataLastSyncMs = syncMs;
}

export function setPlayerDataDetectedTime(detectedMs: number) {
  UPLOADING.progress.state.playerDataLastDetectedMs = detectedMs;
}


