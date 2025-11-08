<script lang="ts">
  import { getClassIcon, tooltip } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { settings, SETTINGS } from "$lib/settings-store";
  import { getDpsPlayers } from "$lib/stores/live-meter-store.svelte";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import { historyDpsPlayerColumns } from "$lib/history-columns";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import PercentFormat from "$lib/components/percent-format.svelte";
  import getDisplayName from "$lib/name-display";

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

  // Track density mode
  let density = $derived(settings.state.accessibility.density ?? "comfortable");

  // Helper functions for density-based styling
  let isCompact = $derived(density === "compact");
  let isMedium = $derived(density === "medium");
  // removed unused isComfortable derived value
</script>

<div class="relative flex flex-col gap-2 overflow-hidden rounded-lg ring-1 ring-neutral-800/60 bg-neutral-950/40 backdrop-blur-sm">
  <table class="w-full border-collapse overflow-hidden">
    <thead>
      <tr class="bg-neutral-900/70">
        <th class="{isCompact ? 'px-2 py-1 text-[10px]' : isMedium ? 'px-2.5 py-1.5 text-[11px]' : 'px-3 py-2 text-[11px]'} text-left font-medium uppercase tracking-wide text-neutral-400">Player</th>
        {#each visiblePlayerColumns as col (col.key)}
          <th class="{isCompact ? 'px-2 py-1 text-[10px]' : isMedium ? 'px-2.5 py-1.5 text-[11px]' : 'px-3 py-2 text-[11px]'} text-right font-medium uppercase tracking-wide text-neutral-400">{col.header}</th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each dpsData as player (player.uid)}
        {@const isLocalPlayer = player.name.includes("You")}
        {@const displayName = getDisplayName({
          player: {
            uid: player.uid,
            name: player.name,
            className: player.className
          },
          showYourNameSetting: SETTINGS_YOUR_NAME,
          showOthersNameSetting: SETTINGS_OTHERS_NAME,
          isLocalPlayer
        })}
        {@const className = isLocalPlayer ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? player.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? player.className : ""}
        <tr
          class="relative bg-neutral-900/50 hover:bg-neutral-800/70 transition-colors cursor-pointer {isCompact ? 'h-7' : isMedium ? 'h-10' : 'h-14'} {isCompact ? 'text-[11px]' : isMedium ? 'text-[12px]' : 'text-[13px]'} group"
          onclick={() => goto(`/live/dps/skills?playerUid=${player.uid}`)}
        >
          <td class="{isCompact ? 'px-2 py-1' : isMedium ? 'px-2.5 py-2' : 'px-3 py-3'} relative z-10">
            <div class="flex items-center h-full {isCompact ? 'gap-1' : isMedium ? 'gap-1.5' : 'gap-2'}">
              <img
                class="{isCompact ? 'size-4' : isMedium ? 'size-5' : 'size-6'} object-contain"
                src={getClassIcon(className)}
                alt="Class icon"
                {@attach tooltip(() => `${player.className}${player.classSpecName ? ' - ' + player.classSpecName : ''}`)}
              />
              {#if player.abilityScore > 0}
                {#if SETTINGS.live.general.state.shortenAbilityScore}
                  <span class="text-neutral-400 tabular-nums"><AbbreviatedNumber num={player.abilityScore} /></span>
                {:else}
                  <span class="text-neutral-400 tabular-nums">{player.abilityScore}</span>
                {/if}
              {/if}
              <span class="truncate font-medium text-neutral-200">{displayName || `#${player.uid}`}</span>
            </div>
          </td>
          {#each visiblePlayerColumns as col (col.key)}
            <td class="{isCompact ? 'px-2 py-1' : isMedium ? 'px-2.5 py-2' : 'px-3 py-3'} text-right relative z-10 tabular-nums font-medium text-neutral-300">
              {#if col.key === 'totalDmg'}
                {#if SETTINGS.live.general.state.shortenDps}
                  <AbbreviatedNumber num={player.totalDmg} />
                {:else}
                  {player.totalDmg.toLocaleString()}
                {/if}
              {:else if col.key === 'dps'}
                {#if SETTINGS.live.general.state.shortenDps}
                  <AbbreviatedNumber num={player.dps} />
                {:else}
                  {Math.round(player.dps).toLocaleString()}
                {/if}
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
