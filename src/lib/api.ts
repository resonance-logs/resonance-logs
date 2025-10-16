import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { commands } from "./bindings";

// Type definitions for event payloads
export type HeaderInfo = {
  totalDps: number;
  totalDmg: number;
  elapsedMs: number
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

export type SkillsUpdatePayload = {
  playerUid: number;
  skillsWindow: SkillsWindow;
};

// Event listener functions
export const onEncounterUpdate = (handler: (event: { payload: EncounterUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("encounter-update", handler);

export const onDpsPlayersUpdate = (handler: (event: { payload: PlayersWindow }) => void): Promise<UnlistenFn> =>
  listen("dps-players-update", handler);

export const onHealPlayersUpdate = (handler: (event: { payload: PlayersWindow }) => void): Promise<UnlistenFn> =>
  listen("heal-players-update", handler);

export const onDpsSkillsUpdate = (handler: (event: { payload: SkillsUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("dps-skills-update", handler);

export const onHealSkillsUpdate = (handler: (event: { payload: SkillsUpdatePayload }) => void): Promise<UnlistenFn> =>
  listen("heal-skills-update", handler);

export const onResetEncounter = (handler: () => void): Promise<UnlistenFn> =>
  listen("reset-encounter", handler);

export const onPauseEncounter = (handler: (event: { payload: boolean }) => void): Promise<UnlistenFn> =>
  listen("pause-encounter", handler);

// Command wrappers (still using generated bindings)
export const resetEncounter = () => commands.resetEncounter();
export const togglePauseEncounter = () => commands.togglePauseEncounter();
export const enableBlur = () => commands.enableBlur();
export const disableBlur = () => commands.disableBlur();
