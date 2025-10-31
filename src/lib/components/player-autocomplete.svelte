<script lang="ts">
	import { onMount } from 'svelte';
	import { commands } from '$lib/bindings';

	let {
		value = $bindable(''),
		placeholder = 'Filter by player name...',
		disabled = false,
		onSelect = () => {},
		id
	}: {
		value: string;
		placeholder?: string;
		disabled?: boolean;
		onSelect: (playerName: string) => void;
		id?: string;
	} = $props();

	let showDropdown = $state(false);
	let filteredPlayerNames = $state<string[]>([]);
	let isLoading = $state(false);

	async function handleInput() {
		const trimmedValue = value.trim();

		// Only query if 3 or more characters
		if (trimmedValue.length < 3) {
			filteredPlayerNames = [];
			showDropdown = false;
			isLoading = false;
			return;
		}

		isLoading = true;
		try {
			const res = await commands.getPlayerNamesFiltered(trimmedValue);
			if (res.status === 'ok') {
				filteredPlayerNames = res.data.names ?? [];
				showDropdown = filteredPlayerNames.length > 0;
			} else {
				console.error('Failed to load player names:', res.error);
				filteredPlayerNames = [];
				showDropdown = false;
			}
		} catch (error) {
			console.error('Error loading player names:', error);
			filteredPlayerNames = [];
			showDropdown = false;
		} finally {
			isLoading = false;
		}
	}

	function selectPlayer(playerName: string) {
		// Clear the input after selecting a player
		value = '';
		showDropdown = false;
		filteredPlayerNames = [];
		onSelect(playerName);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			showDropdown = false;
		}
	}

	function handleFocus() {
		if (value.trim().length >= 3) {
			handleInput();
		}
	}

	function handleBlur() {
		// Delay hiding dropdown to allow click on dropdown items
		setTimeout(() => {
			showDropdown = false;
		}, 200);
	}

	// Close dropdown when clicking outside
	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.player-autocomplete-container')) {
			showDropdown = false;
		}
	}

	onMount(() => {
		document.addEventListener('click', handleClickOutside);
		return () => {
			document.removeEventListener('click', handleClickOutside);
		};
	});
</script>

<div class="player-autocomplete-container relative">
	<input
		type="text"
		bind:value={value}
		oninput={handleInput}
		onfocus={handleFocus}
		onblur={handleBlur}
		onkeydown={handleKeydown}
		{placeholder}
		{disabled}
		{id}
		class="w-full px-3 py-2 bg-neutral-800 border border-neutral-700 rounded text-neutral-300 placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
	/>

	{#if isLoading}
		<div class="absolute z-10 w-full mt-1 bg-neutral-800 border border-neutral-700 rounded shadow-lg px-3 py-2">
			<div class="text-neutral-500 text-sm">Loading...</div>
		</div>
	{:else if showDropdown && filteredPlayerNames.length > 0}
		<div class="absolute z-10 w-full mt-1 bg-neutral-800 border border-neutral-700 rounded shadow-lg max-h-48 overflow-y-auto">
			{#each filteredPlayerNames as playerName}
				<button
					onclick={() => selectPlayer(playerName)}
					class="w-full px-3 py-2 text-left text-neutral-300 hover:bg-neutral-700 focus:bg-neutral-700 focus:outline-none transition-colors"
				>
					{playerName}
				</button>
			{/each}
		</div>
	{:else if value.trim().length >= 3 && !isLoading && filteredPlayerNames.length === 0}
		<div class="absolute z-10 w-full mt-1 bg-neutral-800 border border-neutral-700 rounded shadow-lg px-3 py-2">
			<div class="text-neutral-500 text-sm">No players found</div>
		</div>
	{/if}
</div>

<style>
	.player-autocomplete-container {
		/* ensure the container establishes a positioning context */
		position: relative;
	}
</style>
