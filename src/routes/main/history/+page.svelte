<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { commands } from '$lib/bindings';
	import type { EncounterSummaryDto } from '$lib/bindings';

	let encounters = $state<EncounterSummaryDto[]>([]);
	let errorMsg = $state<string | null>(null);

	// Filters
	let minDmg = $state<number | null>(null);

	// Pagination
	const PAGE_SIZE = 10;
	let page = $state(0); // 0-indexed, page 0 = newest
	let totalCount = $state(0);

	// Auto-refresh (only on page 0)
	let autoRefresh = $state(true);
	const REFRESH_MS = 10000; // 10s

async function loadEncounters(p: number = page) {
	try {
		const offset = p * PAGE_SIZE;
		const res = await commands.getRecentEncounters(PAGE_SIZE, offset);

		if (res.status === 'ok') {
			encounters = res.data.rows ?? [];
			totalCount = res.data.totalCount ?? 0;
			errorMsg = null;
			page = p;
			console.debug('get_recent_encounters response', res.data);
		} else {
			throw new Error(String(res.error));
		}
	} catch (e) {
		console.error('loadEncounters error', e);
		errorMsg = String(e);
		encounters = [];
		totalCount = 0;
	}
}

onMount(() => {
    console.log('History page mounted, loading encounters...');
    loadEncounters(0);
    const id = setInterval(() => {
        if (autoRefresh && page === 0) {
            console.log('Auto-refreshing encounters...');
            loadEncounters(0);
        }
    }, REFRESH_MS);
    return () => clearInterval(id);
});

	function applyFilters(list: EncounterSummaryDto[]) {
		return list.filter((enc) => {
				if (minDmg != null && enc.totalDmg < minDmg) return false;
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
			// Refresh current page. If it becomes empty and we're not on page 0, step back one page.
			await loadEncounters(page);
			if (encounters.length === 0 && page > 0) {
				await loadEncounters(page - 1);
			}
		} else {
			alert(`Failed to delete: ${res.error}`);
		}
	}

</script>

<div class="p-4">
	<div class="flex items-center gap-3 mb-3">
		<label for="minDmg" class="text-sm">Min DMG</label>
		<input id="minDmg" type="number" bind:value={minDmg} placeholder="0" class="w-24 border rounded px-2 py-1" />
		<label for="autoRefresh" class="text-sm">Auto-refresh</label>
		<input id="autoRefresh" type="checkbox" bind:checked={autoRefresh} />
		<button class="ml-auto px-3 py-1 bg-neutral-800 rounded" onclick={() => loadEncounters(page)}>Refresh</button>
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
						<button class="px-2 py-1 mr-2 bg-neutral-700 rounded" onclick={() => onView(enc)}>View</button>
						<button class="px-2 py-1 bg-red-700 rounded" onclick={() => onDelete(enc)}>Delete</button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>

	<!-- Pagination controls -->
	<div class="flex justify-end mt-3">
		<div class="flex items-center gap-2">
			<button class="px-3 py-1 bg-neutral-700 rounded" onclick={() => { if (page > 0) loadEncounters(page - 1); }} disabled={page === 0}>Previous</button>
			<div class="text-sm px-2">Page {page + 1} / {Math.max(1, Math.ceil(totalCount / PAGE_SIZE))}</div>
			<button class="px-3 py-1 bg-neutral-700 rounded" onclick={() => { if ((page + 1) * PAGE_SIZE < totalCount) loadEncounters(page + 1); }} disabled={(page + 1) * PAGE_SIZE >= totalCount}>Next</button>
		</div>
	</div>

	<!-- Debug / counts -->
	<div class="mt-2 text-sm text-neutral-400">
		Showing {encounters.length} encounter(s) on this page — total {totalCount}.
		{#if encounters.length === 0}
			<div class="text-yellow-400 mt-1">No encounters returned for this page.</div>
		{/if}
	</div>
</div>

