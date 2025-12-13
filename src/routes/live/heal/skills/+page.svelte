<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { settings, SETTINGS } from "$lib/settings-store";
  import { commands } from "$lib/bindings";
  import { onHealSkillsUpdate } from "$lib/api";
  import type { Event as TauriEvent } from "@tauri-apps/api/event";
  import type { SkillsWindow, SkillsUpdatePayload } from "$lib/api";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import { historyDpsSkillColumns } from "$lib/column-data"; // Use same structure as DPS for consistency
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import PercentFormat from "$lib/components/percent-format.svelte";

  const playerUid: string = page.url.searchParams.get("playerUid") ?? "-1";

  let healSkillBreakdownWindow: SkillsWindow = $state({
    currPlayer: [],
    skillRows: [],
  });
  let unlisten: (() => void) | null = null;

  // Optimize derived calculations to avoid recalculation on every render
  let maxSkillValue = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state.live.general.showYourName);
  let SETTINGS_OTHERS_NAME = $state(settings.state.live.general.showOthersName);
  let SETTINGS_SHORTEN_DPS = $state(settings.state.live.general.shortenDps);

  // Table customization settings for skills
  let tableSettings = $derived(SETTINGS.live.tableCustomization.state);
  let customThemeColors = $derived(
    SETTINGS.accessibility.state.customThemeColors,
  );

  // Sorting settings for skills
  let sortKey = $derived(SETTINGS.live.sorting.healSkills.state.sortKey);
  let sortDesc = $derived(SETTINGS.live.sorting.healSkills.state.sortDesc);
  let columnOrder = $derived(SETTINGS.live.columnOrder.healSkills.state.order);

  // Handle column header click for sorting
  function handleSort(key: string) {
    if (SETTINGS.live.sorting.healSkills.state.sortKey === key) {
      SETTINGS.live.sorting.healSkills.state.sortDesc =
        !SETTINGS.live.sorting.healSkills.state.sortDesc;
    } else {
      SETTINGS.live.sorting.healSkills.state.sortKey = key;
      SETTINGS.live.sorting.healSkills.state.sortDesc = true;
    }
  }

  // Sorted skill data based on settings
  let sortedSkillRows = $derived.by(() => {
    const data = [...healSkillBreakdownWindow.skillRows];
    data.sort((a, b) => {
      const aVal = (a as Record<string, unknown>)[sortKey] ?? 0;
      const bVal = (b as Record<string, unknown>)[sortKey] ?? 0;
      if (typeof aVal === "number" && typeof bVal === "number") {
        return sortDesc ? bVal - aVal : aVal - bVal;
      }
      return 0;
    });
    return data;
  });

  // Update maxSkillValue when data changes
  $effect(() => {
    maxSkillValue = healSkillBreakdownWindow.skillRows.reduce(
      (max, p) => (p.totalDmg > max ? p.totalDmg : max),
      0,
    );
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state.live.general.showYourName;
    SETTINGS_OTHERS_NAME = settings.state.live.general.showOthersName;
    SETTINGS_SHORTEN_DPS = settings.state.live.general.shortenDps;
  });

  // Get visible columns based on settings and column order
  let visibleSkillColumns = $derived.by(() => {
    const visible = historyDpsSkillColumns.filter(
      (col) => settings.state.live.heal.skillBreakdown[col.key],
    );
    return visible.sort((a, b) => {
      const aIdx = columnOrder.indexOf(a.key);
      const bIdx = columnOrder.indexOf(b.key);
      return aIdx - bIdx;
    });
  });

  let isDestroyed = false;

  async function subscribePlayerSkills() {
    if (isDestroyed) return;
    try {
      // Subscribe and get initial data
      const result = await commands.subscribePlayerSkills(
        parseInt(playerUid),
        "heal",
      );
      if (isDestroyed) return;
      if (result.status === "ok") {
        healSkillBreakdownWindow = result.data;
      } else {
        console.error("Failed to subscribe to player skills:", result.error);
      }

      // Set up websocket listener for updates
      const unlistenFn = await onHealSkillsUpdate(
        (event: TauriEvent<SkillsUpdatePayload>) => {
          if (isDestroyed) return;
          // Only update if this is the correct player
          if (event.payload.playerUid.toString() === playerUid) {
            healSkillBreakdownWindow = event.payload.skillsWindow;
          }
        },
      );

      if (isDestroyed) {
        unlistenFn();
      } else {
        unlisten = unlistenFn;
      }
    } catch (error) {
      console.error("Failed to subscribe to player skills:", error);
    }
  }

  async function unsubscribePlayerSkills() {
    isDestroyed = true;
    try {
      // Remove websocket listener first
      if (unlisten) {
        unlisten();
        unlisten = null;
      }

      // Unsubscribe from backend
      await commands.unsubscribePlayerSkills(parseInt(playerUid), "heal");
    } catch (error) {
      console.error("Failed to unsubscribe from player skills:", error);
    }
  }

  onMount(() => {
    isDestroyed = false;
    subscribePlayerSkills();

    return () => {
      unsubscribePlayerSkills();
    };
  });
</script>

<svelte:window oncontextmenu={() => window.history.back()} />

<div class="relative flex flex-col">
  <table class="w-full border-collapse">
    {#if tableSettings.skillShowHeader}
      <thead class="z-1 sticky top-0">
        <tr
          class="bg-popover/60"
          style="height: {tableSettings.skillHeaderHeight}px;"
        >
          <th
            class="px-2 py-1 text-left font-medium uppercase tracking-wider"
            style="font-size: {tableSettings.skillHeaderFontSize}px; color: {tableSettings.skillHeaderTextColor};"
            >Skill</th
          >
          {#each visibleSkillColumns as col (col.key)}
            <th
              class="px-2 py-1 text-right font-medium uppercase tracking-wider cursor-pointer select-none hover:bg-muted/40 transition-colors"
              style="font-size: {tableSettings.skillHeaderFontSize}px; color: {tableSettings.skillHeaderTextColor};"
              onclick={() => handleSort(col.key)}
            >
              <span class="inline-flex items-center gap-1 justify-end">
                {col.header}
                {#if sortKey === col.key}
                  <span class="text-primary">{sortDesc ? "▼" : "▲"}</span>
                {/if}
              </span>
            </th>
          {/each}
        </tr>
      </thead>
    {/if}
    <tbody>
      {#each sortedSkillRows as skill (skill.name)}
        {@const currPlayer = healSkillBreakdownWindow.currPlayer[0]}
        {#if currPlayer}
          {@const className = currPlayer.name.includes("You")
            ? SETTINGS_YOUR_NAME !== "Hide Your Name"
              ? currPlayer.className
              : ""
            : SETTINGS_OTHERS_NAME !== "Hide Others' Name"
              ? currPlayer.className
              : ""}
          <tr
            class="relative hover:bg-muted/60 transition-colors bg-background/40"
            style="height: {tableSettings.skillRowHeight}px; font-size: {tableSettings.skillFontSize}px;"
          >
            <td
              class="px-2 py-1 relative z-10"
              style="color: {customThemeColors.tableTextColor};"
            >
              <div class="flex items-center gap-1 h-full">
                <span class="truncate">{skill.name}</span>
              </div>
            </td>
            {#each visibleSkillColumns as col (col.key)}
              <td
                class="px-2 py-1 text-right relative z-10"
                style="color: {customThemeColors.tableTextColor};"
              >
                {#if col.key === "totalDmg"}
                  {#if SETTINGS_SHORTEN_DPS}
                    <AbbreviatedNumber
                      num={skill.totalDmg}
                      suffixFontSize={tableSettings.skillAbbreviatedFontSize}
                      suffixColor={customThemeColors.tableAbbreviatedColor}
                    />
                  {:else}
                    {col.format(skill[col.key] ?? 0)}
                  {/if}
                {:else if col.key === "dmgPct"}
                  <PercentFormat
                    val={skill.dmgPct}
                    fractionDigits={0}
                    suffixFontSize={tableSettings.skillAbbreviatedFontSize}
                    suffixColor={customThemeColors.tableAbbreviatedColor}
                  />
                {:else if col.key === "critRate" || col.key === "critDmgRate" || col.key === "luckyRate" || col.key === "luckyDmgRate"}
                  <PercentFormat
                    val={skill[col.key]}
                    suffixFontSize={tableSettings.skillAbbreviatedFontSize}
                    suffixColor={customThemeColors.tableAbbreviatedColor}
                  />
                {:else}
                  {col.format(skill[col.key] ?? 0)}
                {/if}
              </td>
            {/each}
            <TableRowGlow
              isSkill={true}
              {className}
              classSpecName={currPlayer.classSpecName}
              percentage={SETTINGS.live.general.state.relativeToTopHealSkill
                ? maxSkillValue > 0
                  ? (skill.totalDmg / maxSkillValue) * 100
                  : 0
                : skill.dmgPct}
            />
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>
