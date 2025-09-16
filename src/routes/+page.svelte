<script lang="ts">
    import {
        invoke
    } from "@tauri-apps/api/core";
    import { onDestroy, onMount } from 'svelte';
    import type {
        DPSWindow
    } from "$lib/bindings";
    import {
        DPSRows
    } from "./(live)/models";

    let interval: number;
    onMount(() => {
        interval = setInterval(fetchData, 200); // then every 200ms
    });
    onDestroy(() => {
        clearInterval(interval);
    });

    let dpsRows: DPSRows = new DPSRows();
    let dpsWindowPayload: DPSWindow;

    async function fetchData() {
        try {
            dpsWindowPayload = await invoke<DPSWindow>("get_damage_row");
            console.log(+Date.now(), dpsWindowPayload);
            dpsRows.damageRows = dpsWindowPayload.damageRows
        } catch (e) {
            console.error('Error fetching data:', e);
        }
    }
</script>

<main class="container">
    <h1>DPS Meter</h1>

    <table class="table-auto border-collapse border border-gray-300">
        <thead>
        <tr class="bg-gray-200">
            <th class="border border-gray-300 px-2 py-1" style="width:5%">#</th>
            <th class="border border-gray-300 px-2 py-1" style="width:15%">UID</th>
            <th class="border border-gray-300 px-2 py-1" style="width:10%">Name</th>
            <th class="border border-gray-300 px-2 py-1" style="width:5%">ilvl</th>
            <th class="border border-gray-300 px-2 py-1" style="width:10%">Class</th>
            <th class="border border-gray-300 px-2 py-1" style="width:20%">Damage Dealt</th>
            <th class="border border-gray-300 px-2 py-1" style="width:10%">DPS</th>
            <th class="border border-gray-300 px-2 py-1" style="width:5%">%</th>
        </tr>
        </thead>
        <tbody>
        {#each dpsRows.damageRows as player, index}
            <tr class="odd:bg-white even:bg-gray-50 hover:bg-g  ray-100">
                <td class="border border-gray-300 px-2 py-1">{index}</td>
                <td class="border border-gray-300 px-2 py-1">{player.uid}</td>
                <td class="border border-gray-300 px-2 py-1">{player.name || '??'}</td>
                <td class="border border-gray-300 px-2 py-1">{player.abilityScore || '??'}</td>
                <td class="border border-gray-300 px-2 py-1">{player.class || '??'}</td>
                <td class="border border-gray-300 px-2 py-1">{player.totalDamage}</td>
                <td class="border border-gray-300 px-2 py-1">{player.dps.toFixed(2)}</td>
                <td class="border border-gray-300 px-2 py-1">{((Number(player.totalDamage) / Number(dpsWindowPayload.totalDamage)) * 100.0).toFixed(2) + '%'}</td>
            </tr>
        {/each}
        </tbody>
    </table>
</main>


