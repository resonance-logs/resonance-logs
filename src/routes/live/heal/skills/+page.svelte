<script lang="ts">
  import { onMount } from "svelte";
  import { getClassIcon } from "$lib/utils.svelte";
  import { page } from "$app/state";
  import { settings, SETTINGS } from "$lib/settings-store";
  import { commands } from "$lib/bindings";
  import { onHealSkillsUpdate } from "$lib/api";
  import type { Event as TauriEvent } from "@tauri-apps/api/event";
  import type { SkillsWindow, SkillsUpdatePayload } from "$lib/api";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import { historyDpsSkillColumns } from "$lib/history-columns"; // Use same structure as DPS for consistency
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import PercentFormat from "$lib/components/percent-format.svelte";

  const playerUid: string = page.url.searchParams.get("playerUid") ?? "-1";

  let healSkillBreakdownWindow: SkillsWindow = $state({ currPlayer: [], skillRows: [] });
  let unlisten: (() => void) | null = null;

  // Optimize derived calculations to avoid recalculation on every render
  let maxSkillValue = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state.live.general.showYourName);
  let SETTINGS_OTHERS_NAME = $state(settings.state.live.general.showOthersName);
  let SETTINGS_SHORTEN_DPS = $state(settings.state.live.general.shortenDps);

  // Update maxSkillValue when data changes
  $effect(() => {
    maxSkillValue = healSkillBreakdownWindow.skillRows.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0);
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state.live.general.showYourName;
    SETTINGS_OTHERS_NAME = settings.state.live.general.showOthersName;
    SETTINGS_SHORTEN_DPS = settings.state.live.general.shortenDps;
  });

  // Get visible columns based on settings - use same structure as DPS but for healing data
  let visibleSkillColumns = $derived.by(() => {
    return historyDpsSkillColumns.filter(col => settings.state.live.heal.skillBreakdown[col.key]);
  });

  async function subscribePlayerSkills() {
    try {
      // Subscribe and get initial data
      const result = await commands.subscribePlayerSkills(parseInt(playerUid), "heal");
      if (result.status === "ok") {
        healSkillBreakdownWindow = result.data;
      } else {
        console.error("Failed to subscribe to player skills:", result.error);
      }

      // Set up websocket listener for updates
      unlisten = await onHealSkillsUpdate((event: TauriEvent<SkillsUpdatePayload>) => {
        // Only update if this is the correct player
        if (event.payload.playerUid.toString() === playerUid) {
          healSkillBreakdownWindow = event.payload.skillsWindow;
        }
      });
    } catch (error) {
      console.error("Failed to subscribe to player skills:", error);
    }
 }

  async function unsubscribePlayerSkills() {
    try {
      // Unsubscribe from backend
      await commands.unsubscribePlayerSkills(parseInt(playerUid), "heal");

      // Remove websocket listener
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
    } catch (error) {
      console.error("Failed to unsubscribe from player skills:", error);
    }
 }

  onMount(() => {
    subscribePlayerSkills();

    return () => {
      unsubscribePlayerSkills();
    };
  });
</script>

<svelte:window oncontextmenu={() => window.history.back()} />

<div class="relative flex flex-col">
  <table class="w-full border-collapse">
    <thead class="z-1 sticky top-0">
      <tr class="bg-neutral-800">
        <th class="px-2 py-1 text-left text-xs font-medium uppercase tracking-wider text-neutral-400 text-left">Skill</th>
        {#each visibleSkillColumns as col (col.key)}
          <th class="px-2 py-1 text-right text-xs font-medium uppercase tracking-wider text-neutral-400 text-right">{col.header}</th>
        {/each}
      </tr>
    </thead>
    <tbody class="bg-neutral-900">
      {#each healSkillBreakdownWindow.skillRows as skill (skill.name)}
        {@const currPlayer = healSkillBreakdownWindow.currPlayer[0]}
        {#if currPlayer}
          {@const className = currPlayer.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" ? currPlayer.className : ""}
          <tr
            class="relative border-t border-neutral-800 hover:bg-neutral-800 transition-colors h-6 text-xs"
          >
            <td class="px-2 py-1 text-xs text-neutral-300 relative z-10">
              <div class="flex items-center gap-1 h-full">
                <span class="truncate">{skill.name}</span>
              </div>
            </td>
            {#each visibleSkillColumns as col (col.key)}
              <td class="px-2 py-1 text-right text-xs text-neutral-300 relative z-10">
                {#if col.key === 'totalDmg'}
                  {#if SETTINGS_SHORTEN_DPS}
                    <AbbreviatedNumber num={skill.totalDmg} />
                  {:else}
                    {col.format(skill[col.key] ?? 0)}
                  {/if}
                {:else if col.key === 'dmgPct'}
                  <PercentFormat val={skill.dmgPct} fractionDigits={0} />
                {:else if col.key === 'critRate' || col.key === 'critDmgRate' || col.key === 'luckyRate' || col.key === 'luckyDmgRate'}
                  <PercentFormat val={skill[col.key]} />
                {:else}
                  {col.format(skill[col.key] ?? 0)}
                {/if}
              </td>
            {/each}
            <TableRowGlow className={className} percentage={SETTINGS.live.general.state.relativeToTopHealSkill ? maxSkillValue > 0 ? (skill.totalDmg / maxSkillValue) * 100 : 0 : skill.dmgPct} />
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>