<script lang="ts">
    import {
        invoke
    } from "@tauri-apps/api/core";
    import {
        onDestroy,
        onMount
    } from 'svelte';
    import type {
        DPSRow,
        DPSWindow,
        SkillRow
    } from "$lib/bindings";
    import "../../../app.css";

    let interval: number;
    onMount(() => {
        interval = setInterval(fetchData, 200); // then every 200ms
    });
    onDestroy(() => {
        clearInterval(interval);
    });

    let dpsRows: DPSRow[] = [];
    let dpsWindowPayload: DPSWindow;

    // store the clicked player
    let selectedPlayerAndSkills: [DPSRow, SkillRow[]] | null = null;

    function selectRow(playerAndSkill: [DPSRow, SkillRow[]]) {
        selectedPlayerAndSkills = playerAndSkill;
    }

    function clearSelection(event: MouseEvent) {
        event.preventDefault(); // stop default context menu
        selectedPlayerAndSkills = null;
    }

    async function fetchData() {
        try {
            dpsWindowPayload = await invoke<DPSWindow>("get_damage_row");
            console.log(+Date.now(), dpsWindowPayload);
            dpsRows = dpsWindowPayload.damageRows
        } catch (e) {
            console.error('Error fetching data:', e);
        }
    }
</script>

<main class="container">
    <h1>DPS Meter</h1>

    {#if selectedPlayerAndSkills == null}
        <!-- First table -->
        <table class="table-auto border-collapse border border-gray-300 w-full">
            <thead>
            <tr class="bg-gray-200">
                <th class="border border-gray-300 px-2 py-1" style="width:5%">#</th>
                <th class="border border-gray-300 px-2 py-1" style="width:15%">UID</th>
                <th class="border border-gray-300 px-2 py-1" style="width:10%">Class</th>
                <th class="border border-gray-300 px-2 py-1" style="width:5%">ilvl</th>
                <th class="border border-gray-300 px-2 py-1" style="width:10%">Name</th>
                <th class="border border-gray-300 px-2 py-1" style="width:20%">DMG</th>
                <th class="border border-gray-300 px-2 py-1" style="width:10%">DPS</th>
                <th class="border border-gray-300 px-2 py-1" style="width:5%">D%</th>
            </tr>
            </thead>
            <tbody>
            {#each dpsRows as player, index}
                <tr class="odd:bg-white even:bg-gray-50 hover:bg-gray-100 cursor-pointer"
                    on:click={() => selectRow([player, player.skills])}
                > <!-- TODO: skills become static after clicking on player, something wrong about reactivity, maybe just have a rust command for each of these component views -->
                    <td class="border border-gray-300 px-2 py-1">{index + 1}</td>
                    <td class="border border-gray-300 px-2 py-1">{player.uid}</td>
                    <td class="border border-gray-300 px-2 py-1">{player.class || '??'}</td>
                    <td class="border border-gray-300 px-2 py-1">{player.abilityScore || '??'}</td>
                    <td class="border border-gray-300 px-2 py-1">{player.name || '??'}</td>
                    <td class="border border-gray-300 px-2 py-1">{player.totalDamage}</td>
                    <td class="border border-gray-300 px-2 py-1">{player.dps.toFixed()}</td>
                    <td class="border border-gray-300 px-2 py-1">{((Number(player.totalDamage) / Number(dpsWindowPayload.totalDamage)) * 100.0).toFixed(2) + '%'}</td>
                </tr>
            {/each}
            </tbody>
        </table>
    {:else}
    <!-- Second table only shows if a player is selected -->
        <table
                class="table-auto border-collapse border border-gray-300 w-full mt-4"
                on:contextmenu={clearSelection}
        >
            <thead>
            <tr class="bg-gray-200">
                <th class="border border-gray-300 px-2 py-1" style="width:5%">#</th>
                <th class="border border-gray-300 px-2 py-1" style="width:10%">Skill Name</th>
                <th class="border border-gray-300 px-2 py-1" style="width:20%">DMG</th>
                <th class="border border-gray-300 px-2 py-1" style="width:10%">DPS</th>
                <th class="border border-gray-300 px-2 py-1" style="width:5%">D%</th>
            </tr>
            </thead>
            <tbody>
                <tr class="bg-gray-300">
                    <td class="border border-gray-300 px-2 py-1 text-center" colspan={2}>{selectedPlayerAndSkills[0].name || '??'}</td>
                    <td class="border border-gray-300 px-2 py-1">{selectedPlayerAndSkills[0].totalDamage}</td>
                    <td class="border border-gray-300 px-2 py-1">{selectedPlayerAndSkills[0].dps.toFixed()}</td>
                    <td class="border border-gray-300 px-2 py-1">{((Number(selectedPlayerAndSkills[0].totalDamage) / Number(dpsWindowPayload.totalDamage)) * 100.0).toFixed(2) + '%'}</td>
                </tr>
            {#each selectedPlayerAndSkills[1] as skill, index}
                <tr class="bg-white">
                    <td class="border border-gray-300 px-2 py-1">{index + 1}</td>
                    <td class="border border-gray-300 px-2 py-1">{@html skill.name}</td>
                    <td class="border border-gray-300 px-2 py-1">{skill.totalDamage}</td>
                    <td class="border border-gray-300 px-2 py-1">{skill.dps.toFixed()}</td>
                    <td class="border border-gray-300 px-2 py-1">{((Number(skill.totalDamage) / Number(selectedPlayerAndSkills[0].totalDamage)) * 100.0).toFixed(2) + '%'}</td>
                </tr>
            {/each}
            </tbody>
        </table>
    {/if}
</main>
