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

// Event listener functions
export const onEncounterUpdate = (handler: (event: { payload: EncounterUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("encounter-update", handler);

export const onPlayersUpdate = (handler: (event: { payload: PlayersUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("players-update", handler);

export const onSkillsUpdate = (handler: (event: { payload: SkillsUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("skills-update", handler);

export const onBossDeath = (handler: (event: { payload: BossDeathPayload }) => void): Promise<UnlistenFn> =>
  listen("boss-death", handler);

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
import type { Result } from "./bindings";

export const resetEncounter = (): Promise<Result<null, string>> => commands.resetEncounter();
export const togglePauseEncounter = (): Promise<Result<null, string>> => commands.togglePauseEncounter();
export const enableBlur = (): Promise<void> => commands.enableBlur();
export const disableBlur = (): Promise<void> => commands.disableBlur();

// New: toggle boss-only DPS filtering on the backend
export const setBossOnlyDps = (enabled: boolean): Promise<void> => invoke("set_boss_only_dps", { enabled });
