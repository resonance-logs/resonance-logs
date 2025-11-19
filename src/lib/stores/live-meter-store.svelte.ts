    /**
 * @file This file contains the store for the live meter data.
 * It uses `@tauri-store/svelte` to create persistent stores for the DPS, heal, and tanked player data.
 */
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

export function generateMockPlayers(): PlayersWindow {
    const classes = [
        { name: "Stormblade", spec: "Iaido" },
        { name: "Frost Mage", spec: "Icicle" },
        { name: "Wind Knight", spec: "Vanguard" },
        { name: "Verdant Oracle", spec: "Smite" },
        { name: "Heavy Guardian", spec: "Earthfort" },
        { name: "Marksman", spec: "Wildpack" },
        { name: "Shield Knight", spec: "Recovery" },
        { name: "Beat Performer", spec: "Dissonance" },
    ];

    const playerRows = classes.map((cls, idx) => ({
        uid: idx + 1,
        name: cls.name,
        className: cls.name,
        classSpecName: cls.spec,
        abilityScore: Math.floor(Math.random() * 5000) + 1000,
        totalDmg: Math.floor(Math.random() * 10000000) + 1000000,
        dps: Math.floor(Math.random() * 5000000) + 500000,
        dmgPct: Math.random() * 15 + 1,
        critRate: Math.random() * 80 + 10,
        critDmgRate: Math.random() * 60 + 20,
        luckyRate: Math.random() * 30 + 5,
        luckyDmgRate: Math.random() * 20 + 5,
        hits: Math.floor(Math.random() * 5000) + 500,
        hitsPerMinute: Math.floor(Math.random() * 200) + 50
    }));

    // Sort by totalDmg descending
    playerRows.sort((a, b) => b.totalDmg - a.totalDmg);

    return { playerRows };
}

// Load mock data on store initialization
// setTimeout(() => {
//     setDpsPlayers(generateMockPlayers());
//     setHealPlayers(generateMockPlayers());
//     setTankedPlayers(generateMockPlayers());
// }, 1);
