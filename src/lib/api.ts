import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { commands } from "./bindings";

// Type definitions for event payloads
export type HeaderInfo = {
  totalDps: number;
  totalDmg: number;
  elapsedMs: number;
  fightStartTimestampMs: number; // Unix timestamp when fight started
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

export type MetricType = "dps" | "heal";

export type PlayersUpdatePayload = {
  metricType: MetricType;
  playersWindow: PlayersWindow;
};

export type SkillsUpdatePayload = {
  metricType: MetricType;
  playerUid: number;
  skillsWindow: SkillsWindow;
};

// Event listener functions
export const onEncounterUpdate = (handler: (event: { payload: EncounterUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("encounter-update", handler);

export const onPlayersUpdate = (handler: (event: { payload: PlayersUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("players-update", handler);

export const onSkillsUpdate = (handler: (event: { payload: SkillsUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("skills-update", handler);

// Convenience functions for specific skill updates
export const onDpsSkillsUpdate = (handler: (event: { payload: SkillsUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("skills-update", (event) => {
    if (event.payload.metricType === "dps") {
      handler(event);
    }
  });

export const onHealSkillsUpdate = (handler: (event: { payload: SkillsUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("skills-update", (event) => {
    if (event.payload.metricType === "heal") {
      handler(event);
    }
  });

export const onResetEncounter = (handler: () => void): Promise<UnlistenFn> =>
  listen("reset-encounter", handler);

export const onPauseEncounter = (handler: (event: { payload: boolean }) => void): Promise<UnlistenFn> =>
  listen("pause-encounter", handler);

// Command wrappers (still using generated bindings)
export const resetEncounter = () => commands.resetEncounter();
export const togglePauseEncounter = () => commands.togglePauseEncounter();
export const enableBlur = () => commands.enableBlur();
export const disableBlur = () => commands.disableBlur();
