/**
 * @file This file contains type definitions for event payloads and functions for interacting with the backend.
 *
 * @packageDocumentation
 */
import { listen, type UnlistenFn, type Event } from "@tauri-apps/api/event";
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
  currentSegmentType: 'boss' | 'trash' | null;
  currentSegmentName: string | null;
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

export type DamageEvent = {
  timestampMs: number;
  attackerId: number;
  targetId: number;
  targetName: string | null;
  targetMonsterTypeId: number | null;
  amount: number;
  isBossTarget: boolean;
  isKillingBlow: boolean;
};

export type Segment = {
  id: number;
  segmentType: 'boss' | 'trash';
  bossEntityId: number | null;
  bossMonsterTypeId: number | null;
  bossName: string | null;
  startedAtMs: number;
  endedAtMs: number | null;
  totalDamage: number;
  hitCount: number;
  events: DamageEvent[];
};

export type DungeonLog = {
  sceneId: number | null;
  sceneName: string | null;
  combatState: 'idle' | 'inCombat';
  segments: Segment[];
};

// Event listener functions
export const onEncounterUpdate = (handler: (event: Event<EncounterUpdatePayload>) => void): Promise<UnlistenFn> =>
  listen<EncounterUpdatePayload>("encounter-update", handler);

export const onPlayersUpdate = (handler: (event: Event<PlayersUpdatePayload>) => void): Promise<UnlistenFn> =>
  listen<PlayersUpdatePayload>("players-update", handler);

export const onSkillsUpdate = (handler: (event: Event<SkillsUpdatePayload>) => void): Promise<UnlistenFn> =>
  listen<SkillsUpdatePayload>("skills-update", handler);

export const onBossDeath = (handler: (event: Event<BossDeathPayload>) => void): Promise<UnlistenFn> =>
  listen<BossDeathPayload>("boss-death", handler);

export const onSceneChange = (handler: (event: Event<SceneChangePayload>) => void): Promise<UnlistenFn> =>
  listen<SceneChangePayload>("scene-change", handler);

export const onDungeonLogUpdate = (handler: (event: Event<DungeonLog>) => void): Promise<UnlistenFn> =>
  listen<DungeonLog>("log-update", handler);

// Convenience: factory to create metric-filtered listeners
export const makeSkillsUpdateFilter =
  (metric: MetricType) =>
  (handler: (event: Event<SkillsUpdatePayload>) => void): Promise<UnlistenFn> =>
    listen<SkillsUpdatePayload>("skills-update", (event) => {
      if (event.payload.metricType === metric) handler(event);
    });

export const onDpsSkillsUpdate = makeSkillsUpdateFilter("dps");
export const onHealSkillsUpdate = makeSkillsUpdateFilter("heal");
export const onTankedSkillsUpdate = makeSkillsUpdateFilter("tanked");

export const onResetEncounter = (handler: () => void): Promise<UnlistenFn> =>
  listen("reset-encounter", handler);

export type PlayerMetricsResetPayload = {
  segmentName?: string | null;
};
export const onResetPlayerMetrics = (handler: (event: Event<PlayerMetricsResetPayload>) => void): Promise<UnlistenFn> =>
  listen<PlayerMetricsResetPayload>("reset-player-metrics", handler);

export const onPauseEncounter = (handler: (event: Event<boolean>) => void): Promise<UnlistenFn> =>
  listen<boolean>("pause-encounter", handler);

// Command wrappers (still using generated bindings)
import type { Result } from "./bindings";

export const resetEncounter = (): Promise<Result<null, string>> => commands.resetEncounter();
export const togglePauseEncounter = (): Promise<Result<null, string>> => commands.togglePauseEncounter();
export const resetPlayerMetrics = (): Promise<Result<null, string>> => commands.resetPlayerMetrics();
export const enableBlur = (): Promise<void> => commands.enableBlur();
export const disableBlur = (): Promise<void> => commands.disableBlur();

export const setDungeonSegmentsEnabled = (enabled: boolean): Promise<void> =>
  invoke("set_dungeon_segments_enabled", { enabled });

export const getDungeonLog = (): Promise<DungeonLog> => invoke("get_dungeon_log");

export const getEncounterSegments = (encounterId: number): Promise<Segment[]> =>
  invoke("get_encounter_segments", { encounterId });
