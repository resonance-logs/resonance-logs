<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { commands } from '$lib/bindings';
	import type { EncounterSummaryDto } from '$lib/bindings';
	import { getClassIcon, tooltip } from '$lib/utils.svelte';

	let encounters = $state<EncounterSummaryDto[]>([]);
	let errorMsg = $state<string | null>(null);

	// Pagination
	let pageSize = $state(10);
	let page = $state(0); // 0-indexed, page 0 = newest
	let totalCount = $state(0);
	let isRefreshing = $state(false);

	// Class mapping function
	function getClassName(classId: number | null): string {
		if (!classId) return "";
		const classMap: Record<number, string> = {
			1: "Stormblade",
			2: "Frost Mage",
			4: "Wind Knight",
			5: "Verdant Oracle",
			9: "Heavy Guardian",
			11: "Marksman",
			12: "Shield Knight",
			13: "Beat Performer",
		};
		return classMap[classId] ?? "";
	}

async function loadEncounters(p: number = page) {
	isRefreshing = true;
	try {
		const offset = p * pageSize;
		const res = await commands.getRecentEncounters(pageSize, offset);
		console.log(res)

		if (res.status === 'ok') {
			encounters = res.data.rows ?? [];
			totalCount = res.data.totalCount ?? 0;
			errorMsg = null;
			page = p;
		} else {
			throw new Error(String(res.error));
		}
	} catch (e) {
		console.error('loadEncounters error', e);
		errorMsg = String(e);
		encounters = [];
		totalCount = 0;
	} finally {
		isRefreshing = false;
	}
}

onMount(() => {
    console.log('History page mounted, loading encounters...');
    loadEncounters(0);
});

	function fmtDuration(startMs: number, endMs?: number | null) {
		const end = endMs ?? Date.now();
		const secs = Math.max(0, Math.round((end - startMs) / 1000));
		const m = Math.floor(secs / 60);
		const s = secs % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	function fmtDate(ms: number) {
		try {
			const date = new Date(ms);
			return date.toLocaleDateString('en-CA'); // YYYY-MM-DD format
		} catch { return String(ms); }
	}

	function fmtTime(ms: number) {
		try {
			const date = new Date(ms);
			return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
		} catch { return String(ms); }
	}

	async function onView(enc: EncounterSummaryDto) {
			goto(`/main/history/${enc.id}`);
		}

</script>

<div class="p-6">
	{#if errorMsg}
		<div class="text-red-400 mb-3 text-sm">{errorMsg}</div>
	{/if}

	<div class="overflow-x-auto rounded border border-neutral-700">
		<table class="w-full border-collapse">
			<thead>
				<tr class="bg-neutral-800">
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-neutral-400 w-16">ID</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-neutral-400 w-32">Encounter</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-neutral-400">Players</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-neutral-400 w-24">Duration</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-neutral-400 w-32">Date</th>
					<th class="px-3 py-2.5 text-right w-12">
						<button
							onclick={() => loadEncounters(page)}
							class="text-neutral-400 hover:text-neutral-200 transition-colors"
							disabled={isRefreshing}
							aria-label="Refresh encounters"
						>
							<svg
								class:animate-spin={isRefreshing}
								class="w-4 h-4"
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
							>
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
							</svg>
						</button>
					</th>
				</tr>
			</thead>
			<tbody class="bg-neutral-900">
				{#each encounters as enc (enc.id)}
					<tr
						class="border-t border-neutral-800 hover:bg-neutral-800 cursor-pointer transition-colors"
						onclick={() => onView(enc)}
					>
						<td class="px-3 py-2 text-sm text-neutral-300">{enc.id}</td>
							<td class="px-3 py-2 text-sm text-neutral-300">
								{#if enc.bosses.length > 0}
									<div class="flex flex-wrap gap-1">
										{#each enc.bosses as boss}
											<span class="text-xs bg-neutral-800 px-1.5 py-0.5 rounded">{boss}</span>
										{/each}
									</div>
								{:else}
									<span class="text-neutral-500 text-xs">No bosses</span>
								{/if}
							</td>
							<td class="px-3 py-2 text-sm text-neutral-300">
								{#if enc.players.length > 0}
									<div class="flex gap-1 overflow-hidden">
										{#each enc.players as player}
											<img
												class="size-7 object-contain flex-shrink-0"
												src={getClassIcon(getClassName(player.classId))}
												alt="Class icon"
												{@attach tooltip(() => player.name)}
											/>
										{/each}
									</div>
								{:else}
									<span class="text-neutral-500 text-xs">No players</span>
								{/if}
							</td>
							<td class="px-3 py-2 text-sm text-neutral-300">{fmtDuration(enc.startedAtMs, enc.endedAtMs)}</td>
							<td class="px-3 py-2 text-sm text-neutral-300">
								<div class="leading-snug">
									<div>{fmtDate(enc.startedAtMs)}</div>
									<div class="text-xs text-neutral-400">{fmtTime(enc.startedAtMs)}</div>
								</div>
							</td>
						<td></td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

	<!-- Pagination controls -->
	<div class="flex items-center justify-between mt-4 gap-4">
		<div class="flex items-center gap-3 text-sm text-neutral-400">
			<span>Rows per page:</span>
			<input
				type="number"
				bind:value={pageSize}
				min="5"
				max="100"
				class="w-16 px-2 py-1 bg-neutral-800 border border-neutral-700 rounded text-neutral-300"
				onchange={() => loadEncounters(0)}
			/>
			<span>Showing {page * pageSize + 1} - {Math.min((page + 1) * pageSize, totalCount)} of {totalCount}</span>
		</div>

		<div class="flex items-center gap-1 ml-auto">
			<button
				onclick={() => loadEncounters(0)}
				disabled={page === 0}
				class="p-1.5 text-neutral-400 hover:text-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
				aria-label="First page"
			>
				<svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
				</svg>
			</button>
			<button
				onclick={() => loadEncounters(page - 1)}
				disabled={page === 0}
				class="p-1.5 text-neutral-400 hover:text-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
				aria-label="Previous page"
			>
				<svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
				</svg>
			</button>
			<button
				onclick={() => loadEncounters(page + 1)}
				disabled={(page + 1) * pageSize >= totalCount}
				class="p-1.5 text-neutral-400 hover:text-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
				aria-label="Next page"
			>
				<svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</button>
			<button
				onclick={() => loadEncounters(Math.floor((totalCount - 1) / pageSize))}
				disabled={(page + 1) * pageSize >= totalCount}
				class="p-1.5 text-neutral-400 hover:text-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
				aria-label="Last page"
			>
				<svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
				</svg>
			</button>
		</div>
	</div>
</div>

