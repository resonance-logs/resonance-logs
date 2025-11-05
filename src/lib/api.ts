/**
 * @file This file contains type definitions for event payloads and functions for interacting with the backend.
 *
 * @packageDocumentation
 */
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { commands } from "./bindings";

// Type definitions for event payloads
export type BossHealth = {
  uid: number;
  name: string;
  currentHp: number | null;
  maxHp: number | null;
};

export type HeaderInfo = {
  totalDps: number;
  totalDmg: number;
  elapsedMs: number;
  fightStartTimestampMs: number; // Unix timestamp when fight started
  bosses: BossHealth[];
  sceneId: number | null;
  sceneName: string | null;
};

export type PlayerRow = {
  uid: number;
  name: string;
  className: string;
  classSpecName: string;
  abilityScore: number;
  totalDmg: number;
  dps: number;
  dmgPct: number;
  critRate: number;
  critDmgRate: number;
  luckyRate: number;
  luckyDmgRate: number;
  hits: number;
  hitsPerMinute: number
};

export type PlayersWindow = {
  playerRows: PlayerRow[]
};

export type SkillRow = {
  name: string;
  totalDmg: number;
  dps: number;
  dmgPct: number;
  critRate: number;
  critDmgRate: number;
  luckyRate: number;
  luckyDmgRate: number;
  hits: number;
  hitsPerMinute: number
};

export type SkillsWindow = {
  currPlayer: PlayerRow[];
  skillRows: SkillRow[]
};

export type EncounterUpdatePayload = {
  headerInfo: HeaderInfo;
  isPaused: boolean;
};

export type MetricType = "dps" | "heal" | "tanked";

export type PlayersUpdatePayload = {
  metricType: MetricType;
  playersWindow: PlayersWindow;
};

export type SkillsUpdatePayload = {
  metricType: MetricType;
  playerUid: number;
  skillsWindow: SkillsWindow;
};

export type BossDeathPayload = {
  bossName: string;
};

export type SceneChangePayload = {
  sceneName: string;
};

// Event listener functions
export const onEncounterUpdate = (handler: (event: { payload: EncounterUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("encounter-update", handler);

export const onPlayersUpdate = (handler: (event: { payload: PlayersUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("players-update", handler);

export const onSkillsUpdate = (handler: (event: { payload: SkillsUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("skills-update", handler);

export const onBossDeath = (handler: (event: { payload: BossDeathPayload }) => void): Promise<UnlistenFn> =>
  listen("boss-death", handler);

export const onSceneChange = (handler: (event: { payload: SceneChangePayload }) => void): Promise<UnlistenFn> =>
  listen("scene-change", handler);

// Convenience: factory to create metric-filtered listeners
export const makeSkillsUpdateFilter =
  (metric: MetricType) =>
  (handler: (event: { payload: SkillsUpdatePayload }) => void): Promise<UnlistenFn> =>
    listen("skills-update", (event) => {
      if (event.payload.metricType === metric) handler(event);
    });

export const onDpsSkillsUpdate = makeSkillsUpdateFilter("dps");
export const onHealSkillsUpdate = makeSkillsUpdateFilter("heal");
export const onTankedSkillsUpdate = makeSkillsUpdateFilter("tanked");

export const onResetEncounter = (handler: () => void): Promise<UnlistenFn> =>
  listen("reset-encounter", handler);

export const onPauseEncounter = (handler: (event: { payload: boolean }) => void): Promise<UnlistenFn> =>
  listen("pause-encounter", handler);

// Command wrappers (still using generated bindings)
import type { Result, ActorEncounterStatDto as BindingActorEncounterStatDto, SkillsWindow as BindingSkillsWindow } from "./bindings";

export const resetEncounter = (): Promise<Result<null, string>> => commands.resetEncounter();
export const togglePauseEncounter = (): Promise<Result<null, string>> => commands.togglePauseEncounter();
export const enableBlur = (): Promise<void> => commands.enableBlur();
export const disableBlur = (): Promise<void> => commands.disableBlur();

// New: toggle boss-only DPS filtering on the backend
export const setBossOnlyDps = (enabled: boolean): Promise<void> => invoke("set_boss_only_dps", { enabled });

// Attempt (phase) metadata returned by the backend for an encounter
export type Attempt = {
  id: number;
  attemptIndex: number;
  startedAtMs: number;
  endedAtMs: number | null;
  reason: string;
  bossHpStart: number | null;
  bossHpEnd: number | null;
  totalDeaths: number;
};

/**
 * Fetch attempts (phases) for a historical encounter on demand.
 * Returns an array of Attempt DTOs.
 */
export const getEncounterAttempts = async (encounterId: number): Promise<Attempt[]> => {
  try {
    const res = await invoke("get_encounter_attempts", { encounterId });
    return res as Attempt[];
  } catch (e) {
    console.error("Failed to fetch encounter attempts:", e);
    return [];
  }
};

/** Fetch per-attempt actor stats (aggregated from raw events). */
export const getEncounterAttemptActorStats = async (
  encounterId: number,
  attemptIndex: number
): Promise<BindingActorEncounterStatDto[]> => {
  try {
    const res = await invoke("get_encounter_attempt_actor_stats", { encounterId, attemptIndex });
    return res as BindingActorEncounterStatDto[];
  } catch (e) {
    console.error("Failed to fetch attempt actor stats:", e);
    return [];
  }
};

/** Fetch per-attempt player skills (aggregated from raw events). */
export const getEncounterAttemptPlayerSkills = async (
  encounterId: number,
  attemptIndex: number,
  actorId: number,
  skillType: MetricType = "dps"
): Promise<BindingSkillsWindow> => {
  try {
    const res = await invoke("get_encounter_attempt_player_skills", { encounterId, actorId, attemptIndex, skillType });
    return res as BindingSkillsWindow;
  } catch (e) {
    console.error("Failed to fetch attempt player skills:", e);
    return { currPlayer: [], skillRows: [] } as BindingSkillsWindow;
  }
};
