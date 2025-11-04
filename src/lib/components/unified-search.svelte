<script lang="ts">
	import { onMount } from 'svelte';
	import { commands } from '$lib/bindings';

	let {
		value = $bindable(''),
		searchType = $bindable<'boss' | 'player' | 'encounter'>('encounter'),
		placeholder,
		disabled = false,
		availableBossNames = [],
		availableEncounterNames = [],
		onSelect,
		id
		} = $props<{
			value: string;
			searchType: 'boss' | 'player' | 'encounter';
			placeholder?: string;
			disabled?: boolean;
			availableBossNames: string[];
			availableEncounterNames: string[];
			onSelect: (name: string, type: 'boss' | 'player' | 'encounter') => void;
			id?: string;
		}>();

	let showDropdown = $state(false);
	let filteredNames = $state<string[]>([]);
	let isLoading = $state(false);
	let showTypeDropdown = $state(false);

	const computedPlaceholder = $derived(
		placeholder ||
			(searchType === 'boss'
				? 'Search for boss...'
				: searchType === 'encounter'
				? 'Search for encounter...'
				: 'Search for player...')
	);

	async function handleInput() {
		const trimmedValue = value.trim();

		if (searchType === 'boss' || searchType === 'encounter') {
			// Boss filtering - filter locally from available names
			if (trimmedValue === '') {
				filteredNames = [];
				showDropdown = false;
			} else {
				const source = searchType === 'boss' ? availableBossNames : availableEncounterNames;
				filteredNames = source.filter((name: string) =>
					name.toLowerCase().includes(trimmedValue.toLowerCase())
				);
				showDropdown = filteredNames.length > 0;
			}
		} else {
			// Player filtering - query backend with 1-char minimum
			if (trimmedValue.length < 1) {
				filteredNames = [];
				showDropdown = false;
				isLoading = false;
				return;
			}

			isLoading = true;
			try {
				const res = await commands.getPlayerNamesFiltered(trimmedValue);
				if (res.status === 'ok') {
					filteredNames = res.data.names ?? [];
					showDropdown = filteredNames.length > 0;
				} else {
					console.error('Failed to load player names:', res.error);
					filteredNames = [];
					showDropdown = false;
				}
			} catch (error) {
				console.error('Error loading player names:', error);
				filteredNames = [];
				showDropdown = false;
			} finally {
				isLoading = false;
			}
		}
	}

	function selectName(name: string) {
		value = '';
		showDropdown = false;
		filteredNames = [];
		onSelect(name, searchType);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			showDropdown = false;
		}
	}

	function handleFocus() {
		if (value.trim().length >= 1) {
			handleInput();
		}
	}

	function handleBlur() {
		// Delay hiding dropdown to allow click on dropdown items
		setTimeout(() => {
			showDropdown = false;
		}, 200);
	}

	function toggleTypeDropdown() {
		showTypeDropdown = !showTypeDropdown;
	}

	function selectSearchType(type: 'boss' | 'player' | 'encounter') {
		searchType = type;
		showTypeDropdown = false;
		value = '';
		filteredNames = [];
		showDropdown = false;
	}

	// Close dropdown when clicking outside
	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.unified-search-container')) {
			showDropdown = false;
		}
		if (!target.closest('.type-dropdown-container')) {
			showTypeDropdown = false;
		}
	}

	onMount(() => {
		document.addEventListener('click', handleClickOutside);
		return () => {
			document.removeEventListener('click', handleClickOutside);
		};
	});
</script>

<div class="unified-search-container relative flex items-stretch gap-0">
	<!-- Type Selector Dropdown -->
	<div class="type-dropdown-container relative">
		<button
			type="button"
			onclick={toggleTypeDropdown}
			class="h-full px-3 py-1.5 bg-neutral-800 border border-neutral-700 border-r-0 rounded-l text-neutral-300 hover:bg-neutral-750 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed min-w-[90px] flex items-center justify-between gap-2 text-sm"
			{disabled}
		>
			<span class="capitalize">{searchType}</span>
			<svg
				class="w-3.5 h-3.5 transition-transform {showTypeDropdown ? 'rotate-180' : ''}"
				fill="none"
				stroke="currentColor"
				viewBox="0 0 24 24"
			>
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		</button>

		{#if showTypeDropdown}
			<div
				class="absolute left-0 top-full mt-1 z-20 bg-neutral-900 border border-neutral-700 rounded-md shadow-xl overflow-hidden min-w-[120px]"
			>
				<button
					type="button"
					onclick={() => selectSearchType('boss')}
					class="w-full px-3 py-2 text-left text-sm text-neutral-300 hover:bg-neutral-800 focus:bg-neutral-800 focus:outline-none transition-colors {searchType === 'boss' ? 'bg-neutral-800 text-neutral-100' : ''}"
				>
					Boss
				</button>
				<button
					type="button"
					onclick={() => selectSearchType('player')}
					class="w-full px-3 py-2 text-left text-sm text-neutral-300 hover:bg-neutral-800 focus:bg-neutral-800 focus:outline-none transition-colors {searchType === 'player' ? 'bg-neutral-800 text-neutral-100' : ''}"
				>
					Player
				</button>
				<button
					type="button"
					onclick={() => selectSearchType('encounter')}
					class="w-full px-3 py-2 text-left text-sm text-neutral-300 hover:bg-neutral-800 focus:bg-neutral-800 focus:outline-none transition-colors {searchType === 'encounter' ? 'bg-neutral-800 text-neutral-100' : ''}"
				>
					Encounter
				</button>
			</div>
		{/if}
	</div>

	<!-- Search Input -->
	<div class="flex-1 relative">
		<input
			type="text"
			bind:value={value}
			oninput={handleInput}
			onfocus={handleFocus}
			onblur={handleBlur}
			onkeydown={handleKeydown}
			placeholder={computedPlaceholder}
			{disabled}
			{id}
			autocomplete="off"
			class="w-full px-3 py-1.5 text-sm bg-neutral-800 border border-neutral-700 rounded-r text-neutral-300 placeholder-neutral-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
		/>

		{#if isLoading}
			<div
				class="absolute z-10 w-full mt-1 bg-neutral-900 border border-neutral-700 rounded-md shadow-xl px-3 py-2"
			>
				<div class="text-neutral-400 text-sm">Loading...</div>
			</div>
		{:else if showDropdown && filteredNames.length > 0}
			<div
				class="absolute z-10 w-full mt-1 bg-neutral-900 border border-neutral-700 rounded-md shadow-xl max-h-48 overflow-y-auto"
			>
				{#each filteredNames as name}
					<button
						type="button"
						onclick={() => selectName(name)}
						class="w-full px-3 py-2 text-left text-sm text-neutral-300 hover:bg-neutral-800 focus:bg-neutral-800 focus:outline-none transition-colors"
					>
						{name}
					</button>
				{/each}
			</div>
		{:else if searchType === 'player' && value.trim().length >= 1 && !isLoading && filteredNames.length === 0}
			<div
				class="absolute z-10 w-full mt-1 bg-neutral-900 border border-neutral-700 rounded-md shadow-xl px-3 py-2"
			>
				<div class="text-neutral-400 text-sm">No players found</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.unified-search-container {
		/* ensure the container establishes a positioning context */
		position: relative;
	}
</style>
