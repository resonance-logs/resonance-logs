<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { commands } from '$lib/bindings';
	import type { EncounterSummaryDto, EncounterFiltersDto } from '$lib/bindings';
	import { getClassIcon, tooltip } from '$lib/utils.svelte';
	import UnifiedSearch from '$lib/components/unified-search.svelte';
	import FilterChips from '$lib/components/filter-chips.svelte';

	const CLASS_MAP: Record<number, string> = {
		1: 'Stormblade',
		2: 'Frost Mage',
		4: 'Wind Knight',
		5: 'Verdant Oracle',
		9: 'Heavy Guardian',
		11: 'Marksman',
		12: 'Shield Knight',
		13: 'Beat Performer'
	};

	const classOptions = Object.entries(CLASS_MAP).map(([id, name]) => ({
		id: Number(id),
		name
	}));

	function getTodayIso(): string {
		const today = new Date();
		today.setHours(0, 0, 0, 0);
		return today.toISOString().slice(0, 10);
	}

	let encounters = $state<EncounterSummaryDto[]>([]);
	let errorMsg = $state<string | null>(null);

	// Pagination
	let pageSize = $state(10);
	let page = $state(0); // 0-indexed, page 0 = newest
	let totalCount = $state(0);
	let isRefreshing = $state(false);

	// Unified search (boss and player names)
	let availableBossNames = $state<string[]>([]);
	let selectedBosses = $state<string[]>([]);
	let selectedPlayerNames = $state<string[]>([]);
	let searchValue = $state('');
	let searchType = $state<'boss' | 'player'>('boss');
	let isLoadingBossNames = $state(false);

	// Advanced filters
	let showAdvancedFilters = $state(false);
	let selectedClassIds = $state<number[]>([]);
	let classFilterDraft = $state<number[]>([]);
	let dateFrom = $state('');
	let dateTo = $state('');
	let dateFromDraft = $state('');
	let dateToDraft = $state('');
	let dateFromDraftTouched = $state(false);
	let dateToDraftTouched = $state(false);

	let filterMenuRef: HTMLDivElement | null = null;
	let filterButtonRef: HTMLButtonElement | null = null;

	function getClassName(classId: number | null): string {
		if (!classId) return '';
		return CLASS_MAP[classId] ?? '';
	}

	async function loadBossNames() {
		isLoadingBossNames = true;
		try {
			const res = await commands.getUniqueBossNames();
			if (res.status === 'ok') {
				availableBossNames = res.data.names ?? [];
			} else {
				throw new Error(String(res.error));
			}
		} catch (e) {
			console.error('loadBossNames error', e);
			availableBossNames = [];
		} finally {
			isLoadingBossNames = false;
		}
	}

	async function loadEncounters(p: number = page) {
		isRefreshing = true;
		try {
			const offset = p * pageSize;

			let fromTimestamp: number | null = null;
			if (dateFrom) {
				const fromDate = new Date(dateFrom);
				fromDate.setHours(0, 0, 0, 0);
				fromTimestamp = fromDate.getTime();
			}

			let toTimestamp: number | null = null;
			if (dateTo) {
				const toDate = new Date(dateTo);
				toDate.setHours(23, 59, 59, 999);
				toTimestamp = toDate.getTime();
			}

			const filterPayload: EncounterFiltersDto = {
				bossNames: selectedBosses.length > 0 ? selectedBosses : null,
				playerName: null,
				playerNames: selectedPlayerNames.length > 0 ? selectedPlayerNames : null,
				classIds: selectedClassIds.length > 0 ? selectedClassIds : null,
				dateFromMs: fromTimestamp,
				dateToMs: toTimestamp
			};

			const hasFilters =
				filterPayload.bossNames !== null ||
				filterPayload.playerNames !== null ||
				filterPayload.classIds !== null ||
				filterPayload.dateFromMs !== null ||
				filterPayload.dateToMs !== null;

			const res = await commands.getRecentEncountersFiltered(
				pageSize,
				offset,
				hasFilters ? filterPayload : null
			);

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

	function handleSearchSelect(name: string, type: 'boss' | 'player') {
		if (type === 'boss') {
			if (!selectedBosses.includes(name)) {
				selectedBosses = [...selectedBosses, name];
				loadEncounters(0);
			}
		} else {
			if (!selectedPlayerNames.includes(name)) {
				selectedPlayerNames = [...selectedPlayerNames, name];
				loadEncounters(0);
			}
		}
	}

	function removeBossFilter(bossName: string) {
		selectedBosses = selectedBosses.filter((name) => name !== bossName);
		loadEncounters(0);
	}

	function clearAllBossFilters() {
		selectedBosses = [];
		loadEncounters(0);
	}

	function removePlayerNameFilter(playerName: string) {
		selectedPlayerNames = selectedPlayerNames.filter((name) => name !== playerName);
		loadEncounters(0);
	}

	function clearAllPlayerNameFilters() {
		selectedPlayerNames = [];
		loadEncounters(0);
	}

	function toggleAdvancedFilters() {
		if (!showAdvancedFilters) {
			classFilterDraft = [...selectedClassIds];
			const today = getTodayIso();
			dateFromDraft = dateFrom || today;
			dateToDraft = dateTo || today;
			dateFromDraftTouched = dateFrom !== '';
			dateToDraftTouched = dateTo !== '';
		}
		showAdvancedFilters = !showAdvancedFilters;
	}

	function cancelAdvancedFilters() {
		classFilterDraft = [...selectedClassIds];
		const today = getTodayIso();
		dateFromDraft = dateFrom || today;
		dateToDraft = dateTo || today;
		dateFromDraftTouched = dateFrom !== '';
		dateToDraftTouched = dateTo !== '';
		showAdvancedFilters = false;
	}

	function resetDraftFilters() {
		classFilterDraft = [];
		const today = getTodayIso();
		dateFromDraft = today;
		dateToDraft = today;
		dateFromDraftTouched = false;
		dateToDraftTouched = false;
	}

	function toggleDraftClass(classId: number, checked: boolean) {
		if (checked) {
			if (!classFilterDraft.includes(classId)) {
				classFilterDraft = [...classFilterDraft, classId];
			}
		} else {
			classFilterDraft = classFilterDraft.filter((id) => id !== classId);
		}
	}

	function onDraftClassChange(classId: number, event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		toggleDraftClass(classId, input.checked);
	}

	function applyAdvancedFilters() {
		let normalizedFrom = dateFromDraftTouched ? dateFromDraft : '';
		let normalizedTo = dateToDraftTouched ? dateToDraft : '';

		if (normalizedFrom && normalizedTo && normalizedTo < normalizedFrom) {
			[normalizedFrom, normalizedTo] = [normalizedTo, normalizedFrom];
		}

		selectedClassIds = [...classFilterDraft];
		dateFrom = normalizedFrom;
		dateTo = normalizedTo;
		const today = getTodayIso();
		dateFromDraft = normalizedFrom || today;
		dateToDraft = normalizedTo || today;
		dateFromDraftTouched = normalizedFrom !== '';
		dateToDraftTouched = normalizedTo !== '';
		showAdvancedFilters = false;
		loadEncounters(0);
	}

	function clearClassAndDateFilters() {
		selectedClassIds = [];
		classFilterDraft = [];
		dateFrom = '';
		dateTo = '';
		const today = getTodayIso();
		dateFromDraft = today;
		dateToDraft = today;
		dateFromDraftTouched = false;
		dateToDraftTouched = false;
		showAdvancedFilters = false;
		loadEncounters(0);
	}

	function removeClassFilter(classId: number) {
		selectedClassIds = selectedClassIds.filter((id) => id !== classId);
		classFilterDraft = classFilterDraft.filter((id) => id !== classId);
		loadEncounters(0);
	}

	function clearDateFilters() {
		dateFrom = '';
		dateTo = '';
		const today = getTodayIso();
		dateFromDraft = today;
		dateToDraft = today;
		dateFromDraftTouched = false;
		dateToDraftTouched = false;
		loadEncounters(0);
	}

	onMount(() => {
		loadBossNames();
		loadEncounters(0);

		function handleDocumentClick(event: MouseEvent) {
			const target = event.target as HTMLElement;
			if (
				showAdvancedFilters &&
				filterMenuRef &&
				!filterMenuRef.contains(target) &&
				filterButtonRef &&
				!filterButtonRef.contains(target)
			) {
				classFilterDraft = [...selectedClassIds];
				const today = getTodayIso();
				dateFromDraft = dateFrom || today;
				dateToDraft = dateTo || today;
				dateFromDraftTouched = dateFrom !== '';
				dateToDraftTouched = dateTo !== '';
				showAdvancedFilters = false;
			}
		}

		document.addEventListener('click', handleDocumentClick);

		return () => {
			document.removeEventListener('click', handleDocumentClick);
		};
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
			return date.toLocaleDateString('en-CA');
		} catch {
			return String(ms);
		}
	}

	function fmtTime(ms: number) {
		try {
			const date = new Date(ms);
			return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
		} catch {
			return String(ms);
		}
	}

	async function onView(enc: EncounterSummaryDto) {
		goto(`/main/history/${enc.id}`);
	}
</script>

<div class="p-6">
	{#if errorMsg}
		<div class="text-red-400 mb-3 text-sm">{errorMsg}</div>
	{/if}

	<!-- Unified Search Section (Boss or Player) -->
	<div class="mb-6">
		<div class="flex flex-col gap-3">
			<div class="flex items-center gap-3">
				<label for="unified-search" class="text-sm font-medium text-neutral-300">
					Search:
				</label>
				<div class="flex-1 max-w-md">
					<UnifiedSearch
						id="unified-search"
						bind:value={searchValue}
						bind:searchType={searchType}
						availableBossNames={availableBossNames}
						onSelect={handleSearchSelect}
						disabled={isLoadingBossNames}
					/>
				</div>
			</div>
			{#if selectedBosses.length > 0}
				<div>
					<div class="text-xs font-medium text-neutral-400 mb-1">Bosses:</div>
					<FilterChips
						filters={selectedBosses}
						onRemove={removeBossFilter}
						onClearAll={clearAllBossFilters}
					/>
				</div>
			{/if}
			{#if selectedPlayerNames.length > 0}
				<div>
					<div class="text-xs font-medium text-neutral-400 mb-1">Players:</div>
					<FilterChips
						filters={selectedPlayerNames}
						onRemove={removePlayerNameFilter}
						onClearAll={clearAllPlayerNameFilters}
					/>
				</div>
			{/if}
		</div>
	</div>

	<!-- Advanced filters -->
	<div class="mb-6">
		<div class="flex flex-wrap items-end gap-3">
			<div class="relative">
				<button
					bind:this={filterButtonRef}
					onclick={toggleAdvancedFilters}
					class="px-3 py-2 text-sm font-medium rounded bg-neutral-800 border border-neutral-700 text-neutral-200 hover:text-white hover:border-neutral-500 focus:outline-none focus:ring-2 focus:ring-blue-400"
					aria-expanded={showAdvancedFilters}
				>
					Advanced Filters
				</button>
				{#if showAdvancedFilters}
					<div
						bind:this={filterMenuRef}
						class="absolute left-0 z-20 mt-2 w-80 rounded border border-neutral-700 bg-neutral-900 p-4 shadow-xl"
					>
						<div class="space-y-4">
							<div>
								<h4 class="mb-2 text-sm font-semibold text-neutral-100">Classes</h4>
								<div class="grid grid-cols-2 gap-2 max-h-40 overflow-y-auto pr-1">
									{#each classOptions as option}
										<label class="flex items-center gap-2 text-sm text-neutral-200">
											<input
												type="checkbox"
												checked={classFilterDraft.includes(option.id)}
												onchange={(event) => onDraftClassChange(option.id, event)}
												class="size-4 rounded border-neutral-600 bg-neutral-800"
											/>
											<span>{option.name}</span>
										</label>
									{/each}
								</div>
							</div>
							<div class="grid gap-2">
								<h4 class="text-sm font-semibold text-neutral-100">Date Range</h4>
								<label class="text-xs uppercase tracking-wide text-neutral-500">From</label>
								<input
									type="date"
									bind:value={dateFromDraft}
									class="px-3 py-2 rounded border border-neutral-700 bg-neutral-800 text-neutral-200 focus:outline-none focus:ring-2 focus:ring-blue-400"
								/>
								<label class="text-xs uppercase tracking-wide text-neutral-500">To</label>
								<input
									type="date"
									bind:value={dateToDraft}
									class="px-3 py-2 rounded border border-neutral-700 bg-neutral-800 text-neutral-200 focus:outline-none focus:ring-2 focus:ring-blue-400"
								/>
							</div>
							<div class="flex items-center justify-between gap-2 pt-2">
								<button
									onclick={resetDraftFilters}
									class="text-sm text-neutral-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-neutral-500"
								>
									Reset
								</button>
								<div class="flex gap-2">
									<button
										onclick={cancelAdvancedFilters}
										class="px-3 py-1.5 text-sm rounded border border-neutral-700 text-neutral-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-neutral-500"
									>
										Cancel
									</button>
									<button
										onclick={applyAdvancedFilters}
										class="px-3 py-1.5 text-sm font-semibold rounded bg-blue-600 text-white hover:bg-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-400"
									>
										Apply
									</button>
								</div>
							</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
		{#if selectedClassIds.length > 0 || dateFrom || dateTo}
			<div class="mt-3 flex flex-wrap items-center gap-2 text-sm text-neutral-200">
				{#each selectedClassIds as classId}
					<span class="inline-flex items-center gap-1 rounded bg-neutral-800 px-2 py-1">
						{CLASS_MAP[classId]}
						<button
							onclick={() => removeClassFilter(classId)}
							class="text-neutral-400 hover:text-red-300 focus:outline-none"
							aria-label={`Remove ${CLASS_MAP[classId]} class filter`}
						>
							✕
						</button>
					</span>
				{/each}
				{#if dateFrom || dateTo}
					<span class="inline-flex items-center gap-1 rounded bg-neutral-800 px-2 py-1">
						Date: {dateFrom || 'Any'} – {dateTo || 'Any'}
						<button
							onclick={clearDateFilters}
							class="text-neutral-400 hover:text-red-300 focus:outline-none"
							aria-label="Clear date filters"
						>
							✕
						</button>
					</span>
				{/if}
				{#if selectedClassIds.length > 0 || dateFrom || dateTo}
					<button
						onclick={clearClassAndDateFilters}
						class="ml-auto text-sm text-neutral-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-neutral-500"
					>
						Clear advanced filters
					</button>
				{/if}
			</div>
		{/if}
	</div>

	<div class="overflow-x-auto rounded border border-neutral-700">
		<table class="w-full border-collapse">
			<thead>
				<tr class="bg-neutral-800">
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-neutral-400 w-16">ID</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-neutral-400 w-32">Encounter</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-neutral-400 w-[400px]">Players</th>
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
							<td class="px-3 py-2 text-sm text-neutral-300 max-w-[400px]">
								{#if enc.players.length > 0}
									{@const sortedPlayers = [...enc.players].sort((a, b) => {
										const aHasClass = a.classId !== null && a.classId !== undefined && a.classId !== 0;
										const bHasClass = b.classId !== null && b.classId !== undefined && b.classId !== 0;
										if (aHasClass && !bHasClass) return -1;
										if (!aHasClass && bHasClass) return 1;
										return 0;
									})}
									<div class="flex gap-1 items-center">
										{#each sortedPlayers.slice(0, 8) as player}
											<img
												class="size-7 object-contain flex-shrink-0"
												src={getClassIcon(getClassName(player.classId))}
												alt="Class icon"
												{@attach tooltip(() => player.name)}
											/>
										{/each}
										{#if enc.players.length > 8}
											<span class="text-xs text-neutral-400 ml-1">+{enc.players.length - 8} more</span>
										{/if}
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
