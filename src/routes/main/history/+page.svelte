<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { commands } from '$lib/bindings';
	import type { EncounterSummaryDto } from '$lib/bindings';

	let encounters: EncounterSummaryDto[] = [];
	let errorMsg: string | null = null;

	// Filters
	let dateFrom: string = ""; // yyyy-mm-dd
	let dateTo: string = "";
	let minDmg: number | null = null;

	// Auto-refresh
	let autoRefresh = true;
	const REFRESH_MS = 10000; // 10s
	let limit = 200;

	async function loadEncounters() {
		try {
			const res = await commands.getRecentEncounters(limit);
			if (res.status === 'ok') {
				encounters = res.data;
				errorMsg = null;
			} else {
				errorMsg = String(res.error);
			}
		} catch (e) {
			errorMsg = String(e);
		}
	}

	onMount(() => {
		loadEncounters();
		const id = setInterval(() => {
			if (autoRefresh) loadEncounters();
		}, REFRESH_MS);
		return () => clearInterval(id);
	});

	function applyFilters(list: EncounterSummaryDto[]) {
		return list.filter((enc) => {
			if (minDmg != null && enc.totalDmg < minDmg) return false;
			if (dateFrom) {
				const fromMs = new Date(dateFrom).getTime();
				if (enc.startedAtMs < fromMs) return false;
			}
			if (dateTo) {
				// include full day
				const toMs = new Date(dateTo).getTime() + 24 * 60 * 60 * 1000 - 1;
				if ((enc.endedAtMs ?? enc.startedAtMs) > toMs) return false;
			}
			return true;
		});
	}

	function fmtDate(ms: number) {
		try {
			return new Date(ms).toLocaleString();
		} catch { return String(ms); }
	}

	function fmtDuration(startMs: number, endMs?: number | null) {
		const end = endMs ?? Date.now();
		const secs = Math.max(0, Math.round((end - startMs) / 1000));
		const m = Math.floor(secs / 60);
		const s = secs % 60;
		return `${m}m ${s}s`;
	}

	async function onView(enc: EncounterSummaryDto) {
		goto(`/main/history/encounter?encounterId=${enc.id}`);
	}

	async function onDelete(enc: EncounterSummaryDto) {
		if (!confirm(`Delete encounter #${enc.id}? This action cannot be undone.`)) return;
		const res = await commands.deleteEncounter(enc.id);
		if (res.status === 'ok') {
			encounters = encounters.filter(e => e.id !== enc.id);
		} else {
			alert(`Failed to delete: ${res.error}`);
		}
	}

</script>

<div class="p-4">
	<div class="flex items-center gap-3 mb-3">
		<label class="text-sm">From</label>
		<input type="date" bind:value={dateFrom} class="border rounded px-2 py-1" />
		<label class="text-sm">To</label>
		<input type="date" bind:value={dateTo} class="border rounded px-2 py-1" />
		<label class="text-sm">Min DMG</label>
		<input type="number" bind:value={minDmg} placeholder="0" class="w-24 border rounded px-2 py-1" />
		<label class="text-sm">Auto-refresh</label>
		<input type="checkbox" bind:checked={autoRefresh} />
		<button class="ml-auto px-3 py-1 bg-neutral-800 rounded" on:click={() => loadEncounters()}>Refresh</button>
	</div>

	{#if errorMsg}
		<div class="text-red-400 mb-2">{errorMsg}</div>
	{/if}

	<table class="w-full table-fixed border-collapse">
		<thead>
			<tr class="bg-neutral-900 text-left">
				<th class="p-2">ID</th>
				<th class="p-2">Started</th>
				<th class="p-2">Duration</th>
				<th class="p-2 text-right">Total DMG</th>
				<th class="p-2 text-right">Total Heal</th>
				<th class="p-2">Actions</th>
			</tr>
		</thead>
		<tbody>
			{#each applyFilters(encounters) as enc (enc.id)}
				<tr class="hover:bg-neutral-800 border-b">
					<td class="p-2">{enc.id}</td>
					<td class="p-2">{fmtDate(enc.startedAtMs)}</td>
					<td class="p-2">{fmtDuration(enc.startedAtMs, enc.endedAtMs)}</td>
					<td class="p-2 text-right">{enc.totalDmg}</td>
					<td class="p-2 text-right">{enc.totalHeal}</td>
					<td class="p-2">
						<button class="px-2 py-1 mr-2 bg-neutral-700 rounded" on:click={() => onView(enc)}>View</button>
						<button class="px-2 py-1 bg-red-700 rounded" on:click={() => onDelete(enc)}>Delete</button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

