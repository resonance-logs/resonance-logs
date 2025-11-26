<script lang="ts">
  import { getClassIcon, tooltip } from "$lib/utils.svelte";
  import { goto } from "$app/navigation";
  import { settings, SETTINGS } from "$lib/settings-store";
  import { getHealPlayers } from "$lib/stores/live-meter-store.svelte";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import { liveHealPlayerColumns } from "$lib/history-columns";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import PercentFormat from "$lib/components/percent-format.svelte";
  import getDisplayName from "$lib/name-display";

  // Create reactive data reference to avoid recreating table on every render
  let healData = $state(getHealPlayers().playerRows);

  // Update data when store changes
  $effect(() => {
    healData = getHealPlayers().playerRows;
  });

  // Optimize derived calculations to avoid recalculation on every render
  let maxHeal = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state.live.general.showYourName);
  let SETTINGS_OTHERS_NAME = $state(settings.state.live.general.showOthersName);
  let SETTINGS_SHORTEN_DPS = $state(settings.state.live.general.shortenDps);

  // Table customization settings
  let tableSettings = $derived(SETTINGS.live.tableCustomization.state);

  // Update maxHeal when data changes
  $effect(() => {
    const players = getHealPlayers().playerRows;
    maxHeal = players.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0);
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state.live.general.showYourName;
    SETTINGS_OTHERS_NAME = settings.state.live.general.showOthersName;
    SETTINGS_SHORTEN_DPS = settings.state.live.general.shortenDps;
  });

  // Get visible columns based on settings
  let visiblePlayerColumns = $derived.by(() => {
    return liveHealPlayerColumns.filter(col => settings.state.live.heal.players[col.key]);
  });

  

</script>

<div class="relative flex flex-col gap-2 overflow-hidden rounded-lg ring-1 ring-border/60 bg-card/30">
  <table class="w-full border-collapse overflow-hidden">
    {#if tableSettings.showTableHeader}
      <thead>
        <tr class="bg-popover/60" style="height: {tableSettings.tableHeaderHeight}px;">
          <th class="px-3 py-1 text-left font-medium uppercase tracking-wide" style="font-size: {tableSettings.tableHeaderFontSize}px; color: {tableSettings.tableHeaderTextColor};">Player</th>
          {#each visiblePlayerColumns as col (col.key)}
            <th class="px-3 py-1 text-right font-medium uppercase tracking-wide" style="font-size: {tableSettings.tableHeaderFontSize}px; color: {tableSettings.tableHeaderTextColor};">{col.header}</th>
          {/each}
        </tr>
      </thead>
    {/if}
    <tbody>
      {#each healData as player (player.uid)}
        {@const isLocalPlayer = player.name.includes("You")}
        {@const displayName = getDisplayName({
          player: {
            uid: player.uid,
            name: player.name,
            className: player.className,
            classSpecName: player.classSpecName
          },
          showYourNameSetting: SETTINGS_YOUR_NAME,
          showOthersNameSetting: SETTINGS_OTHERS_NAME,
          isLocalPlayer
        })}
        {@const className = isLocalPlayer ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? player.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? player.className : ""}
        <tr
          class="relative bg-background/40 hover:bg-muted/60 transition-colors cursor-pointer group"
          style="height: {tableSettings.playerRowHeight}px; font-size: {tableSettings.playerFontSize}px;"
          onclick={() => goto(`/live/heal/skills?playerUid=${player.uid}`)}
        >
          <td class="px-3 py-1 relative z-10">
            <div class="flex items-center h-full gap-2">
              <img
                style="width: {tableSettings.playerIconSize}px; height: {tableSettings.playerIconSize}px;"
                class="object-contain"
                src={getClassIcon(className)}
                alt="Class icon"
                {@attach tooltip(() => `${player.className}${player.classSpecName ? ' - ' + player.classSpecName : ''}`)}
              />
              {#if player.abilityScore > 0}
                {#if SETTINGS.live.general.state.shortenAbilityScore}
                  <span class="tabular-nums" style="color: {tableSettings.playerTextColor};"><AbbreviatedNumber num={player.abilityScore} suffixFontSize={tableSettings.abbreviatedFontSize} suffixColor={tableSettings.abbreviatedColor} /></span>
                {:else}
                  <span class="tabular-nums" style="color: {tableSettings.playerTextColor};">{player.abilityScore}</span>
                {/if}
              {/if}
              <span class="truncate font-medium" style="color: {tableSettings.playerTextColor};">{displayName || `#${player.uid}`}</span>
            </div>
          </td>
          {#each visiblePlayerColumns as col (col.key)}
            <td class="px-3 py-1 text-right relative z-10 tabular-nums font-medium" style="color: {tableSettings.playerTextColor};">
              {#if col.key === 'totalDmg'}
                    {#if SETTINGS_SHORTEN_DPS}
                      <AbbreviatedNumber num={player.totalDmg} suffixFontSize={tableSettings.abbreviatedFontSize} suffixColor={tableSettings.abbreviatedColor} />
                    {:else}
                      {player.totalDmg.toLocaleString()}
                    {/if}
              {:else if col.key === 'dmgPct'}
                <PercentFormat val={player.dmgPct} fractionDigits={0} suffixFontSize={tableSettings.abbreviatedFontSize} suffixColor={tableSettings.abbreviatedColor} />
              {:else if col.key === 'critRate' || col.key === 'critDmgRate' || col.key === 'luckyRate' || col.key === 'luckyDmgRate'}
                <PercentFormat val={player[col.key]} suffixFontSize={tableSettings.abbreviatedFontSize} suffixColor={tableSettings.abbreviatedColor} />
              {:else}
                {col.format(player[col.key] ?? 0)}
              {/if}
            </td>
          {/each}
          <TableRowGlow className={className} classSpecName={player.classSpecName} percentage={SETTINGS.live.general.state.relativeToTopHealPlayer ? maxHeal > 0 ? (player.totalDmg / maxHeal) * 100 : 0 : player.dmgPct} />
        </tr>
      {/each}
    </tbody>
  </table>
</div>
