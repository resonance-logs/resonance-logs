<script lang="ts">
    import {
        invoke
    } from "@tauri-apps/api/core";
    import { onDestroy, onMount } from 'svelte';
    import type {
        DamageRow
    } from "$lib/bindings";
    import {
        DamageRows
    } from "./(live)/models";

    let interval: number;
    onMount(() => {
        interval = setInterval(fetchData, 200); // then every 200ms
    });
    onDestroy(() => {
        clearInterval(interval);
    });

    let damageRows: DamageRows = new DamageRows();

    async function fetchData() {
        try {
            damageRows = new DamageRows(await invoke<DamageRow[]>("get_damage_row"));
            console.log(+Date.now(), damageRows);

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
            <th class="border border-gray-300 px-2 py-1" style="width:25%">Name</th>
            <th class="border border-gray-300 px-2 py-1" style="width:10%">ilvl</th>
            <th class="border border-gray-300 px-2 py-1" style="width:10%">Class</th>
            <th class="border border-gray-300 px-2 py-1" style="width:20%">Damage Dealt</th>
            <th class="border border-gray-300 px-2 py-1" style="width:15%">DPS</th>
        </tr>
        </thead>
        <tbody>
        {#each damageRows.damageRows as damageRow, index}
            <tr class="odd:bg-white even:bg-gray-50 hover:bg-gray-100">
                <td class="border border-gray-300 px-2 py-1">{index}</td>
                <td class="border border-gray-300 px-2 py-1">{damageRow.uid}</td>
                <td class="border border-gray-300 px-2 py-1">{damageRow.name || '??'}</td>
                <td class="border border-gray-300 px-2 py-1">{damageRow.abilityScore || '??'}</td>
                <td class="border border-gray-300 px-2 py-1">{damageRow.class || '??'}</td>
                <td class="border border-gray-300 px-2 py-1">{damageRow.totalDamage}</td>
                <td class="border border-gray-300 px-2 py-1">{damageRow.dps.toFixed(2)}</td>
            </tr>
        {/each}
        </tbody>

    </table>
</main>


