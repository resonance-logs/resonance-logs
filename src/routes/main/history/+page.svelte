<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { commands } from '$lib/bindings';
	import type { EncounterSummaryDto, EncounterFiltersDto } from '$lib/bindings';
	import { getClassIcon, tooltip, CLASS_MAP } from '$lib/utils.svelte';
	import UnifiedSearch from '$lib/components/unified-search.svelte';
	import FilterChips from '$lib/components/filter-chips.svelte';

	const classOptions = Object.entries(CLASS_MAP).map(([id, name]) => ({
		id: Number(id),
		name
	}));

	let encounters = $state<EncounterSummaryDto[]>([]);
	let errorMsg = $state<string | null>(null);

	// Pagination
	let pageSize = $state(10);
	let page = $state(0); // 0-indexed, page 0 = newest
	let totalCount = $state(0);
	let isRefreshing = $state(false);

	// Unified search (boss, player and encounter names)
	let availableBossNames = $state<string[]>([]);
	let availableEncounterNames = $state<string[]>([]);
	let selectedBosses = $state<string[]>([]);
	let selectedEncounters = $state<string[]>([]);
	let selectedPlayerNames = $state<string[]>([]);
	let searchValue = $state('');
	let searchType = $state<'boss' | 'player' | 'encounter'>('encounter');
	let isLoadingBossNames = $state(false);

	// Class filters
	let selectedClassIds = $state<number[]>([]);
	let showClassDropdown = $state(false);
	let classDropdownRef: HTMLDivElement | null = null;
	let classButtonRef: HTMLButtonElement | null = null;

	function getClassName(classId: number | null): string {
		if (!classId) return '';
		return CLASS_MAP[classId] ?? '';
	}

	async function loadSceneNames() {
		try {
			const res = await commands.getUniqueSceneNames();
			if (res.status === 'ok') {
				availableEncounterNames = res.data.names ?? [];
			} else {
				availableEncounterNames = [];
			}
		} catch (e) {
			console.error('loadSceneNames error', e);
			availableEncounterNames = [];
		}
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

			const filterPayload: EncounterFiltersDto = {
				bossNames: selectedBosses.length > 0 ? selectedBosses : null,
				playerName: null,
				encounterNames: null,
				playerNames: selectedPlayerNames.length > 0 ? selectedPlayerNames : null,
				classIds: selectedClassIds.length > 0 ? selectedClassIds : null,
				dateFromMs: null,
				dateToMs: null
			};

			const hasFilters =
				filterPayload.bossNames !== null ||
				filterPayload.playerNames !== null ||
				filterPayload.classIds !== null;

			const res = await commands.getRecentEncountersFiltered(
				pageSize,
				offset,
				hasFilters ? filterPayload : null
			);

			if (res.status === 'ok') {
				console.log("encounter data", res.data)
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

	function handleSearchSelect(name: string, type: 'boss' | 'player' | 'encounter') {
		if (type === 'boss') {
			if (!selectedBosses.includes(name)) {
				selectedBosses = [...selectedBosses, name];
				loadEncounters(0);
			}
		} else if (type === 'encounter') {
			if (!selectedEncounters.includes(name)) {
				selectedEncounters = [...selectedEncounters, name];
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

	function removeEncounterFilter(sceneName: string) {
		selectedEncounters = selectedEncounters.filter((n) => n !== sceneName);
		loadEncounters(0);
	}

	function clearAllEncounterFilters() {
		selectedEncounters = [];
		loadEncounters(0);
	}

	function toggleClassFilter(classId: number, checked: boolean) {
		if (checked) {
			if (!selectedClassIds.includes(classId)) {
				selectedClassIds = [...selectedClassIds, classId];
			}
		} else {
			selectedClassIds = selectedClassIds.filter((id) => id !== classId);
		}
		loadEncounters(0);
	}

	function removeClassFilter(classId: number) {
		selectedClassIds = selectedClassIds.filter((id) => id !== classId);
		loadEncounters(0);
	}

	function clearAllClassFilters() {
		selectedClassIds = [];
		loadEncounters(0);
	}

	function clearAllFilters() {
		selectedBosses = [];
		selectedPlayerNames = [];
		selectedClassIds = [];
		selectedEncounters = [];
		loadEncounters(0);
	}

    const hasActiveFilters = $derived(
        selectedBosses.length > 0 || selectedPlayerNames.length > 0 || selectedClassIds.length > 0 || selectedEncounters.length > 0
    );

	onMount(() => {
	loadBossNames();
	loadSceneNames();
	loadEncounters(0);

		function handleDocumentClick(event: MouseEvent) {
			const target = event.target as HTMLElement;
			if (
				showClassDropdown &&
				classDropdownRef &&
				!classDropdownRef.contains(target) &&
				classButtonRef &&
				!classButtonRef.contains(target)
			) {
				showClassDropdown = false;
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

<div class="">
	{#if errorMsg}
		<div class="text-red-400 mb-3 text-sm">{errorMsg}</div>
	{/if}

	<!-- Filters Section -->
	<div class="mb-2 space-y-2">
		<!-- Search and Filter Row -->
		<div class="flex items-center gap-2">
			<div class="flex-1 max-w-md">
				<UnifiedSearch
					id="unified-search"
					bind:value={searchValue}
					bind:searchType={searchType}
					availableBossNames={availableBossNames}
					availableEncounterNames={availableEncounterNames}
					onSelect={handleSearchSelect}
					disabled={isLoadingBossNames}
				/>
			</div>

			<!-- Class Filter Dropdown -->
			<div class="relative">
							<button
								bind:this={classButtonRef}
								onclick={() => showClassDropdown = !showClassDropdown}
								class="flex items-center gap-2 px-3 py-1.5 rounded-md border border-border bg-popover hover:bg-muted/40 transition-colors text-sm text-muted-foreground hover:text-foreground"
							>
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
					</svg>
					<span>Classes</span>
				</button>

				{#if showClassDropdown}
					<div
						bind:this={classDropdownRef}
						class="absolute right-0 z-20 mt-2 w-64 rounded-md border border-neutral-700 bg-neutral-900 shadow-xl"
					>
						<div class="p-3 space-y-1">
							{#each classOptions as option}
												<button
													onclick={() => toggleClassFilter(option.id, !selectedClassIds.includes(option.id))}
													class="w-full flex items-center justify-between px-3 py-2 rounded-md text-sm transition-colors {selectedClassIds.includes(option.id) ? 'bg-primary/15 text-primary' : 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
												>
									<span>{option.name}</span>
									{#if selectedClassIds.includes(option.id)}
										<svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
											<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
										</svg>
									{/if}
								</button>
							{/each}

							{#if selectedClassIds.length > 0}
												<button
													onclick={clearAllClassFilters}
													class="w-full px-3 py-2 text-sm text-muted-foreground hover:text-destructive transition-colors text-left"
												>
									Clear class filters
								</button>
							{/if}
						</div>
					</div>
				{/if}
			</div>

			<!-- Clear All Filters Button -->
			{#if hasActiveFilters}
							<button
								onclick={clearAllFilters}
								class="px-3 py-1.5 rounded-md text-sm text-muted-foreground hover:text-destructive transition-colors"
								title="Clear all active filters"
							>
					Clear All
				</button>
			{/if}
		</div>

		<!-- Active Filters Chips -->
		{#if hasActiveFilters}
			<div class="flex flex-wrap items-center gap-1.5">
				{#each selectedBosses as boss}
								<span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] bg-popover text-muted-foreground leading-tight border border-border/60">
									<span class="text-muted-foreground/70">B:</span>
						{boss}
						<button
							onclick={() => removeBossFilter(boss)}
							  class="text-muted-foreground/70 hover:text-destructive transition-colors"
							aria-label={`Remove ${boss} filter`}
						>
							✕
						</button>
					</span>
				{/each}
				{#each selectedPlayerNames as player}
								<span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] bg-popover text-muted-foreground leading-tight border border-border/60">
									<span class="text-muted-foreground/70">P:</span>
						{player}
						<button
							onclick={() => removePlayerNameFilter(player)}
							  class="text-muted-foreground/70 hover:text-destructive transition-colors"
							aria-label={`Remove ${player} filter`}
						>
							✕
						</button>
					</span>
				{/each}
				{#each selectedClassIds as classId}
								<span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] bg-popover text-muted-foreground leading-tight border border-border/60">
									<span class="text-muted-foreground/70">C:</span>
						{CLASS_MAP[classId]}
						<button
							onclick={() => removeClassFilter(classId)}
							  class="text-muted-foreground/70 hover:text-destructive transition-colors"
							aria-label={`Remove ${CLASS_MAP[classId]} filter`}
						>
							✕
						</button>
					</span>
				{/each}
				{#each selectedEncounters as encounter}
								<span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] bg-popover text-muted-foreground leading-tight border border-border/60">
									<span class="text-muted-foreground/70">E:</span>
						{encounter}
						<button
							onclick={() => removeBossFilter(encounter)}
							  class="text-muted-foreground/70 hover:text-destructive transition-colors"
							aria-label={`Remove ${encounter} filter`}
						>
							✕
						</button>
					</span>
				{/each}
			</div>
		{/if}
	</div>

	<div class="overflow-x-auto rounded border border-border/60 bg-card/30">
		<table class="w-full border-collapse" style="min-width: 740px;">
			<thead>
				<tr class="bg-popover/60">
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground w-10">ID</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground w-36">Encounter</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground w-48">Bosses</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground w-[400px]">Players</th>
					<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground w-12">Duration</th>
						<th class="px-3 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground w-48">Date</th>
					<th class="px-3 py-2.5 text-right w-12 text-muted-foreground">
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
			<tbody class="bg-background/40">
				{#each encounters as enc (enc.id)}
					<tr
						class="border-t border-border/40 hover:bg-muted/60 transition-colors"
					>
						<td class="px-3 py-2 text-sm text-muted-foreground cursor-pointer" onclick={() => onView(enc)}>{enc.id}</td>
						<td class="px-3 py-2 text-sm text-muted-foreground cursor-pointer" onclick={() => onView(enc)}>
							{#if enc.sceneName}
								<span class="text-xs bg-muted px-1.5 py-0.5 rounded text-foreground">{enc.sceneName}</span>
							{:else}
								<span class="text-muted-foreground text-xs opacity-70">No scene</span>
							{/if}
						</td>
							<td class="px-3 py-2 text-sm text-muted-foreground cursor-pointer" onclick={() => onView(enc)}>
								{#if enc.bosses.length > 0}
									<div class="flex flex-wrap gap-1">
										{#each enc.bosses as boss}
											<span class="text-xs px-1.5 py-0.5 rounded {boss.isDefeated ? 'text-[oklch(0.65_0.1_145)] bg-muted' : 'bg-muted text-foreground'}">{boss.monsterName}</span>
										{/each}
									</div>
								{:else}
									<span class="text-muted-foreground text-xs opacity-70">No bosses</span>
								{/if}
							</td>
							<td class="px-3 py-2 text-sm text-muted-foreground max-w-[400px] cursor-pointer" onclick={() => onView(enc)}>
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
												{@attach tooltip(() => player.isLocalPlayer ? `${player.name} (You)` : player.name)}
											/>
										{/each}
										{#if enc.players.length > 8}
											<span class="text-xs text-muted-foreground ml-1">+{enc.players.length - 8} more</span>
										{/if}
									</div>
								{:else}
									<span class="text-muted-foreground text-xs opacity-70">No players</span>
								{/if}
							</td>
							<td class="px-3 py-2 text-sm text-muted-foreground cursor-pointer" onclick={() => onView(enc)}>{fmtDuration(enc.startedAtMs, enc.endedAtMs)}</td>
							<td class="px-3 py-2 text-sm text-muted-foreground cursor-pointer" onclick={() => onView(enc)}>
								<div class="leading-snug">
									<div>{fmtDate(enc.startedAtMs)}</div>
									<div class="text-xs text-muted-foreground opacity-70">{fmtTime(enc.startedAtMs)}</div>
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
		<div class="flex items-center gap-3 text-sm text-muted-foreground">
			<span>Rows per page:</span>
					<input
						type="number"
						bind:value={pageSize}
						min="5"
						max="100"
						class="w-16 px-2 py-1 bg-popover border border-border rounded text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
						onchange={() => loadEncounters(0)}
					/>
			<span>Showing {page * pageSize + 1} - {Math.min((page + 1) * pageSize, totalCount)} of {totalCount}</span>
		</div>

		<div class="flex items-center gap-1 ml-auto">
					<button
						onclick={() => loadEncounters(0)}
						disabled={page === 0}
						class="p-1.5 text-muted-foreground hover:text-foreground disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
				aria-label="First page"
			>
				<svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
				</svg>
			</button>
					<button
						onclick={() => loadEncounters(page - 1)}
						disabled={page === 0}
						class="p-1.5 text-muted-foreground hover:text-foreground disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
				aria-label="Previous page"
			>
				<svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
				</svg>
			</button>
					<button
						onclick={() => loadEncounters(page + 1)}
						disabled={(page + 1) * pageSize >= totalCount}
						class="p-1.5 text-muted-foreground hover:text-foreground disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
				aria-label="Next page"
			>
				<svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</button>
					<button
						onclick={() => loadEncounters(Math.floor((totalCount - 1) / pageSize))}
						disabled={(page + 1) * pageSize >= totalCount}
						class="p-1.5 text-muted-foreground hover:text-foreground disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
				aria-label="Last page"
			>
				<svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
				</svg>
			</button>
		</div>
	</div>
</div>
