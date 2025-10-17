import type { PlayersWindow } from "$lib/api";

// Live meter data store using Svelte 5 runes
let dpsPlayers: PlayersWindow = $state({ playerRows: [] });
let healPlayers: PlayersWindow = $state({ playerRows: [] });

// Store functions
export function setDpsPlayers(players: PlayersWindow) {
    dpsPlayers = players;
}

export function setHealPlayers(players: PlayersWindow) {
    healPlayers = players;
}

export function clearMeterData() {
    dpsPlayers = { playerRows: [] };
    healPlayers = { playerRows: [] };
}

// Reactive state getters
export function getDpsPlayers() {
    return dpsPlayers;
}

export function getHealPlayers() {
    return healPlayers;
}
