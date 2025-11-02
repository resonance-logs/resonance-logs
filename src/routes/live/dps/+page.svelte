<script lang="ts">
  import { getClassIcon, tooltip } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { settings, SETTINGS } from "$lib/settings-store";
 import { getDpsPlayers } from "$lib/stores/live-meter-store.svelte";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import { historyDpsPlayerColumns } from "$lib/history-columns";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import PercentFormat from "$lib/components/percent-format.svelte";

  // Create reactive references
  let dpsData = $state(getDpsPlayers().playerRows);

  // Update data when store changes
  $effect(() => {
    dpsData = getDpsPlayers().playerRows;
  });

  // Optimize derived calculations to avoid recalculation on every render
  let maxDamage = $state(0);
 let SETTINGS_YOUR_NAME = $state(settings.state["general"]["showYourName"]);
  let SETTINGS_OTHERS_NAME = $state(settings.state["general"]["showOthersName"]);

  // Update maxDamage when data changes
 $effect(() => {
    const players = getDpsPlayers().playerRows;
    maxDamage = players.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0);
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state["general"]["showYourName"];
    SETTINGS_OTHERS_NAME = settings.state["general"]["showOthersName"];
  });

  // Get visible columns based on settings
  let visiblePlayerColumns = $derived.by(() => {
    return historyDpsPlayerColumns.filter(col => settings.state.live.dps.players[col.key]);
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
      {#each dpsData as player (player.uid)}
        {@const className = player.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? player.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? player.className : ""}
        <tr
          class="relative bg-neutral-900/60 hover:bg-neutral-800/60 transition-all cursor-pointer {isCompactMode ? 'h-7' : 'h-14'} {isCompactMode ? 'text-xs' : 'text-base'} group"
          onclick={() => goto(`/live/dps/skills?playerUid=${player.uid}`)}
        >
          <td class="{isCompactMode ? 'px-2 py-1' : 'px-3 py-3'} {isCompactMode ? 'text-xs' : 'text-base'} text-neutral-200 relative z-10">
            <div class="flex items-center {isCompactMode ? 'gap-1.5' : 'gap-3'} h-full">
              <img
                class="{isCompactMode ? 'size-4' : 'size-6'} object-contain"
                src={getClassIcon(className)}
                alt="Class icon"
                {@attach tooltip(() => `${player.className}${player.classSpecName ? '-' + player.classSpecName : ''}`)}
              />
              <span class="truncate font-medium">
                {#if player.abilityScore > 0}
                  {#if SETTINGS.live.general.state.shortenAbilityScore}
                    <span class="text-neutral-400"><AbbreviatedNumber num={player.abilityScore} /></span>
                  {:else}
                    <span class="text-neutral-400">{player.abilityScore}</span>
                  {/if}
                {/if}
                {player.name || `#${player.uid}`}
              </span>
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
          <TableRowGlow className={className} percentage={SETTINGS.live.general.state.relativeToTopDPSPlayer ? (maxDamage > 0 ? (player.totalDmg / maxDamage) * 100 : 0) : player.dmgPct} />
        </tr>
      {/each}
    </tbody>
  </table>
</div>
