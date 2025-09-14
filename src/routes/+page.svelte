<script lang="ts">
    import {
        invoke
    } from "@tauri-apps/api/core";
    import { onDestroy, onMount } from 'svelte';
    import type {
        EncounterInner
    } from "$lib/ts_rs_generated/EncounterInner";

    let interval: number;
    onMount(() => {
        interval = setInterval(fetchData, 200); // then every 200ms
    });
    onDestroy(() => {
        clearInterval(interval);
    });

    let encounterPayload: EncounterInner;

    async function fetchData() {
        try {
            encounterPayload = await invoke<EncounterInner>("get_encounter");
            console.log(+Date.now(), encounterPayload);

        } catch (e) {
            console.error('Error fetching data:', e);
        }
    }
</script>

<main class="container">
    <h1>DPS Meter</h1>
    {#if encounterPayload && encounterPayload.entities}
        {#each Object.entries(encounterPayload.entities) as [key, entity], index}
            <p>
                Index: {index} | UID: {entity.uid} | Name: {entity.name || '??'} | Damage Dealt: {entity.damageStats?.damageDealt}
            </p>
        {/each}
    {/if}
</main>
