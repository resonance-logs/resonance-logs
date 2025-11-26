    /**
 * @file This file contains the store for the live meter data.
 * It uses `@tauri-store/svelte` to create persistent stores for the DPS, heal, and tanked player data.
 */
import type { PlayersWindow } from "$lib/api";
import { RuneStore } from '@tauri-store/svelte';
import { generateDummyPlayersWindow } from "$lib/dummy-data";

// Live meter data store using RuneStore with improved cleanup
const dpsPlayersStore = new RuneStore<PlayersWindow>(
    'liveMeterDps',
    { playerRows: [] },
    { autoStart: false, saveOnChange: false }
);

const healPlayersStore = new RuneStore<PlayersWindow>(
    'liveMeterHeal',
    { playerRows: [] },
    { autoStart: false, saveOnChange: false }
);

const tankedPlayersStore = new RuneStore<PlayersWindow>(
    'liveMeterTanked',
    { playerRows: [] },
    { autoStart: false, saveOnChange: false }
);

// Cleanup function for stores - RuneStore handles its own cleanup
// but we can ensure proper state reset
export function cleanupStores() {
    // The existing clearMeterData function already handles proper cleanup
    clearMeterData();
}

// Export store functions
export function setDpsPlayers(players: PlayersWindow) {
    dpsPlayersStore.state.playerRows = players.playerRows;
}

export function setHealPlayers(players: PlayersWindow) {
    healPlayersStore.state.playerRows = players.playerRows;
}

export function setTankedPlayers(players: PlayersWindow) {
    tankedPlayersStore.state.playerRows = players.playerRows;
}

export function clearMeterData() {
    dpsPlayersStore.state.playerRows = [];
    healPlayersStore.state.playerRows = [];
    tankedPlayersStore.state.playerRows = [];
}

export function getDpsPlayers() {
    return dpsPlayersStore.state;
}

export function getHealPlayers() {
    return healPlayersStore.state;
}

export function getTankedPlayers() {
    return tankedPlayersStore.state;
}


// Live dungeon segments state
let liveDungeonLog = $state<import('$lib/api').DungeonLog | null>(null);

export function setLiveDungeonLog(log: import('$lib/api').DungeonLog | null) {
    liveDungeonLog = log;
}

export function getLiveDungeonLog() {
    return liveDungeonLog;
}

export function clearLiveDungeonLog() {
    liveDungeonLog = null;
}

// Dummy data injection
export function injectDummyData() {
    const dummyData = generateDummyPlayersWindow();
    setDpsPlayers(dummyData);
    setHealPlayers(dummyData);
    setTankedPlayers(dummyData);
}
