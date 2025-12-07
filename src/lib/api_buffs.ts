import { invoke } from "@tauri-apps/api/core";

export interface BuffEventDto {
    startMs: number;
    endMs: number;
    durationMs: number;
    stackCount: number;
}

export interface BuffInfoDto {
    buffId: number;
    buffName: string;
    events: BuffEventDto[];
}

export interface EntityBuffsDto {
    entityUid: number;
    entityName: string;
    buffs: BuffInfoDto[];
}

export interface EncounterBuffEventDto {
    startMs: number;
    endMs: number;
    durationMs: number;
    stackCount: number;
}

export interface EncounterBuffDto {
    buffId: number;
    buffName: string;
    events: EncounterBuffEventDto[];
}

export interface EncounterEntityBuffsDto {
    entityUid: number;
    entityName: string;
    buffs: EncounterBuffDto[];
}

export async function getLiveBuffs(): Promise<EntityBuffsDto[]> {
    return await invoke("get_live_buffs");
}

export async function getEncounterBuffs(encounterId: number): Promise<EncounterEntityBuffsDto[]> {
    return await invoke("get_encounter_buffs", { encounterId });
}
