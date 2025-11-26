<script lang="ts">
  import { commands } from "$lib/bindings";
  import { settings, SETTINGS } from "$lib/settings-store";
  import type { SkillsWindow, SkillsUpdatePayload } from "$lib/api";
  import { onTankedSkillsUpdate } from "$lib/api";
  import type { Event as TauriEvent } from "@tauri-apps/api/event";
  import { getTankedPlayers } from "$lib/stores/live-meter-store.svelte";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import { liveTankedSkillColumns } from "$lib/column-data"; // Use tanked structure for consistency
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import PercentFormat from "$lib/components/percent-format.svelte";

  let skillsWindow: SkillsWindow | null = $state(null);
  let unlisten: (() => void) | null = null;

  // Get the playerUid from the URL query parameters
  const playerUid = Number($page.url.searchParams.get("playerUid"));

  // Optimize derived calculations to avoid recalculation on every render
  let maxTakenSkill = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state.live.general.showYourName);
  let SETTINGS_OTHERS_NAME = $state(settings.state.live.general.showOthersName);
  let SETTINGS_SHORTEN_TPS = $state(settings.state.live.general.shortenTps);
  let SETTINGS_RELATIVE_TO_TOP_TANKED_SKILL = $state(settings.state.live.general.relativeToTopTankedSkill);

  // Table customization settings for skills
  let tableSettings = $derived(SETTINGS.live.tableCustomization.state);

  // Update maxTakenSkill when data changes
  $effect(() => {
    maxTakenSkill = skillsWindow?.skillRows.reduce((max, s) => (s.totalDmg > max ? s.totalDmg : max), 0) ?? 0;
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state.live.general.showYourName;
    SETTINGS_OTHERS_NAME = settings.state.live.general.showOthersName;
    SETTINGS_SHORTEN_TPS = settings.state.live.general.shortenTps;
    SETTINGS_RELATIVE_TO_TOP_TANKED_SKILL = settings.state.live.general.relativeToTopTankedSkill;
  });

  // Get visible columns based on settings - use same structure as DPS but for tanked data
  let visibleSkillColumns = $derived.by(() => {
    return liveTankedSkillColumns.filter(col => settings.state.live.tanked.skills[col.key]);
  });

  onMount(() => {
    let isDestroyed = false;

    if (playerUid) {
      // Fetch initial skills data
      commands.getPlayerSkills(playerUid, "tanked").then(result => {
        if (isDestroyed) return;
        if (result.status === "ok") {
          skillsWindow = result.data;
        }
      }).catch(e => {
        console.error("Failed to load tanked skills:", e);
      });

      // Listen for updates
        onTankedSkillsUpdate((event: TauriEvent<SkillsUpdatePayload>) => {
        if (isDestroyed) return;
        if (event.payload.playerUid === playerUid) {
          skillsWindow = event.payload.skillsWindow;
        }
      }).then(fn => {
        if (isDestroyed) {
          fn();
        } else {
          unlisten = fn;
        }
      });
    }

    return () => {
      isDestroyed = true;
      if (unlisten) {
        unlisten();
      }
    };
  });

  // Get players list for breadcrumb info
  let tankedPlayers: any[] = $state([]);
  let currentPlayer: any = $state(null);

  // Update players list when store changes
  $effect(() => {
    tankedPlayers = getTankedPlayers().playerRows;
    currentPlayer = tankedPlayers.find((p) => p.uid === playerUid);
  });
</script>

<!-- Breadcrumb to go back to the main table -->
{#if currentPlayer}
  {@const className = currentPlayer.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currentPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? currentPlayer.className : ""}
  <div
    class="sticky top-0 z-10 flex h-8 w-full items-center gap-2 bg-popover/60 px-2 text-xs"
    style="background-color: {`color-mix(in srgb, ${className ? `var(--class-color-${className.toLowerCase().replace(/\s+/g, '-')})` : '#6b7280'} 30%, transparent)`};"
  >
    <button class="underline" onclick={() => goto("/live/tanked")}>Back</button>
    <span class="font-bold">{currentPlayer.name}</span>
    <span>{currentPlayer.classSpecName}</span>
      <span class="ml-auto">
      <span class="text-xs">Total: </span>
      {#if SETTINGS_SHORTEN_TPS}
        <AbbreviatedNumber num={currentPlayer.totalDmg} suffixFontSize={tableSettings.skillAbbreviatedFontSize} suffixColor={tableSettings.skillAbbreviatedColor} />
      {:else}
        {currentPlayer.totalDmg.toLocaleString()}
      {/if}
    </span>
  </div>
{/if}

<div class="relative flex flex-col">
  <table class="w-full border-collapse">
    {#if tableSettings.skillShowHeader}
      <thead class="z-1 sticky top-0">
        <tr class="bg-popover/60" style="height: {tableSettings.skillHeaderHeight}px;">
          <th class="px-2 py-1 text-left font-medium uppercase tracking-wider" style="font-size: {tableSettings.skillHeaderFontSize}px; color: {tableSettings.skillHeaderTextColor};">Skill</th>
          {#each visibleSkillColumns as col (col.key)}
            <th class="px-2 py-1 text-right font-medium uppercase tracking-wider" style="font-size: {tableSettings.skillHeaderFontSize}px; color: {tableSettings.skillHeaderTextColor};">{col.header}</th>
          {/each}
        </tr>
      </thead>
    {/if}
    <tbody>
      {#each skillsWindow?.skillRows as skill (skill.name)}
        {@const className = currentPlayer?.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currentPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" && currentPlayer ? currentPlayer.className : ""}
        <tr
          class="relative border-t border-border hover:bg-muted/60 transition-colors bg-background/40"
          style="height: {tableSettings.skillRowHeight}px; font-size: {tableSettings.skillFontSize}px;"
        >
          <td class="px-2 py-1 relative z-10" style="color: {tableSettings.skillTextColor};">
            <div class="flex items-center gap-1 h-full">
              <span class="truncate">{skill.name}</span>
            </div>
          </td>
          {#each visibleSkillColumns as col (col.key)}
            <td class="px-2 py-1 text-right relative z-10" style="color: {tableSettings.skillTextColor};">
              {#if col.key === 'totalDmg'}
                {#if SETTINGS_SHORTEN_TPS}
                  <AbbreviatedNumber num={skill.totalDmg} suffixFontSize={tableSettings.skillAbbreviatedFontSize} suffixColor={tableSettings.skillAbbreviatedColor} />
                {:else}
                  {skill.totalDmg.toLocaleString()}
                {/if}
              {:else if col.key === 'dps'}
                {#if SETTINGS_SHORTEN_TPS}
                  <AbbreviatedNumber num={skill.dps} suffixFontSize={tableSettings.skillAbbreviatedFontSize} suffixColor={tableSettings.skillAbbreviatedColor} />
                {:else}
                  {skill.dps.toFixed(1)}
                {/if}
              {:else if col.key === 'dmgPct'}
                <PercentFormat val={skill.dmgPct} fractionDigits={0} suffixFontSize={tableSettings.skillAbbreviatedFontSize} suffixColor={tableSettings.skillAbbreviatedColor} />
              {:else if col.key === 'critRate' || col.key === 'critDmgRate' || col.key === 'luckyRate' || col.key === 'luckyDmgRate'}
                <PercentFormat val={skill[col.key]} suffixFontSize={tableSettings.skillAbbreviatedFontSize} suffixColor={tableSettings.skillAbbreviatedColor} />
              {:else}
                {col.format(skill[col.key] ?? 0)}
              {/if}
            </td>
          {/each}
          <TableRowGlow className={className} classSpecName={currentPlayer?.classSpecName} percentage={SETTINGS_RELATIVE_TO_TOP_TANKED_SKILL ? (maxTakenSkill > 0 ? (skill.totalDmg / maxTakenSkill) * 100 : 0) : skill.dmgPct} />
        </tr>
      {/each}
    </tbody>
  </table>
</div>