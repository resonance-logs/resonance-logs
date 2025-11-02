<script lang="ts">
  import { getClassIcon } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { settings } from "$lib/settings-store";
  import { getTankedPlayers } from "$lib/stores/live-meter-store.svelte";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import { historyDpsPlayerColumns } from "$lib/history-columns";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import PercentFormat from "$lib/components/percent-format.svelte";

  // Create reactive data reference to avoid recreating table on every render
  let tankedData = $state(getTankedPlayers().playerRows);

  // Update data when store changes
  $effect(() => {
    tankedData = getTankedPlayers().playerRows;
  });

  // Optimize derived calculations to avoid recalculation on every render
  let maxTaken = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state["general"]["showYourName"]);
  let SETTINGS_OTHERS_NAME = $state(settings.state["general"]["showOthersName"]);

  // Update maxTaken when data changes
  $effect(() => {
    const players = getTankedPlayers().playerRows;
    maxTaken = players.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0);
  });

  // Update settings references when settings change
 $effect(() => {
    SETTINGS_YOUR_NAME = settings.state["general"]["showYourName"];
    SETTINGS_OTHERS_NAME = settings.state["general"]["showOthersName"];
  });

  // Get visible columns based on settings - use the same structure as DPS but for tanked data
  let visiblePlayerColumns = $derived.by(() => {
    return historyDpsPlayerColumns.filter(col => settings.state.live.tanked.players[col.key]);
  });

  // Track compact mode
  let isCompactMode = $derived(settings.state.accessibility.compactMode);
</script>

<div class="relative flex flex-col gap-1 overflow-hidden">
  <table class="w-full border-collapse overflow-hidden">
    <thead>
      <tr class="bg-neutral-900/60">
        <th class="{isCompactMode ? 'px-2 py-1 text-[10px]' : 'px-3 py-2 text-xs'} text-left font-medium uppercase tracking-wider text-neutral-500">Player</th>
        {#each visiblePlayerColumns as col (col.key)}
          <th class="{isCompactMode ? 'px-2 py-1 text-[10px]' : 'px-3 py-2 text-xs'} text-right font-medium uppercase tracking-wider text-neutral-500">{col.header}</th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each tankedData as player (player.uid)}
        {@const className = player.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? player.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? player.className : ""}
        <tr
          class="relative bg-neutral-900/60 hover:bg-neutral-800/60 transition-all cursor-pointer {isCompactMode ? 'h-7' : 'h-14'} {isCompactMode ? 'text-xs' : 'text-base'} group"
          onclick={() => goto(`/live/tanked/skills?playerUid=${player.uid}`)}
        >
          <td class="{isCompactMode ? 'px-2 py-1' : 'px-3 py-3'} {isCompactMode ? 'text-xs' : 'text-base'} text-neutral-200 relative z-10">
            <div class="flex items-center {isCompactMode ? 'gap-1.5' : 'gap-3'} h-full">
              <img
                class="{isCompactMode ? 'size-4' : 'size-6'} object-contain"
                src={getClassIcon(className)}
                alt="Class icon"
              />
              <span class="truncate font-medium">{player.name || `#${player.uid}`}</span>
            </div>
          </td>
          {#each visiblePlayerColumns as col (col.key)}
            <td class="{isCompactMode ? 'px-2 py-1' : 'px-3 py-3'} text-right {isCompactMode ? 'text-xs' : 'text-base'} text-neutral-200 relative z-10">
              {#if col.key === 'totalDmg'}
                <AbbreviatedNumber num={player.totalDmg} />
              {:else if col.key === 'dmgPct'}
                <PercentFormat val={player.dmgPct} fractionDigits={0} />
              {:else if col.key === 'critRate' || col.key === 'critDmgRate' || col.key === 'luckyRate' || col.key === 'luckyDmgRate'}
                <PercentFormat val={player[col.key]} />
              {:else}
                {col.format(player[col.key] ?? 0)}
              {/if}
            </td>
          {/each}
          <TableRowGlow className={className} percentage={settings.state.general.relativeToTopDPSPlayer ? (maxTaken > 0 ? (player.totalDmg / maxTaken) * 100 : 0) : player.dmgPct} />
        </tr>
      {/each}
    </tbody>
  </table>
</div>
