import type { PlayersWindow } from "$lib/api";
import { RuneStore } from '@tauri-store/svelte';

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
