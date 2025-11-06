<script lang="ts">
  import { onMount } from "svelte";
  import { getClassIcon } from "$lib/utils.svelte";
  import { page } from "$app/state";
  import { settings, SETTINGS } from "$lib/settings-store";
 import { commands } from "$lib/bindings";
  import { onDpsSkillsUpdate } from "$lib/api";
  import type { SkillsWindow, SkillsUpdatePayload } from "$lib/api";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import { historyDpsSkillColumns } from "$lib/history-columns";
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import PercentFormat from "$lib/components/percent-format.svelte";

  const playerUid: string = page.url.searchParams.get("playerUid") ?? "-1";

  let dpsSkillBreakdownWindow: SkillsWindow = $state({ currPlayer: [], skillRows: [] });
  let unlisten: (() => void) | null = null;

  // Update data when window changes
  $effect(() => {
    // currPlayerData = dpsSkillBreakdownWindow.currPlayer;
    // skillRowsData = dpsSkillBreakdownWindow.skillRows;
  });

  // Optimize derived calculations to avoid recalculation on every render
  let maxSkillValue = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state["general"]["showYourName"]);
  let SETTINGS_OTHERS_NAME = $state(settings.state["general"]["showOthersName"]);

  // Update maxSkillValue when data changes
  $effect(() => {
    maxSkillValue = dpsSkillBreakdownWindow.skillRows.reduce((max, p) => (p.totalDmg > max ? p.totalDmg : max), 0);
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state["general"]["showYourName"];
    SETTINGS_OTHERS_NAME = settings.state["general"]["showOthersName"];
  });

  // Get visible columns based on settings
  let visibleSkillColumns = $derived.by(() => {
    return historyDpsSkillColumns.filter(col => settings.state.live.dps.skillBreakdown[col.key]);
  });

  async function subscribePlayerSkills() {
    try {
      // Subscribe and get initial data
      const result = await commands.subscribePlayerSkills(parseInt(playerUid), "dps");
      if (result.status === "ok") {
        dpsSkillBreakdownWindow = result.data;
      } else {
        console.error("Failed to subscribe to player skills:", result.error);
      }

      // Set up websocket listener for updates
      unlisten = await onDpsSkillsUpdate((event: { payload: SkillsUpdatePayload }) => {
        // Only update if this is the correct player
        if (event.payload.playerUid.toString() === playerUid) {
          dpsSkillBreakdownWindow = event.payload.skillsWindow;
        }
      });
    } catch (error) {
      console.error("Failed to subscribe to player skills:", error);
    }
 }

  async function unsubscribePlayerSkills() {
    try {
      // Unsubscribe from backend
      await commands.unsubscribePlayerSkills(parseInt(playerUid), "dps");

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
      {#each dpsSkillBreakdownWindow.skillRows as skill (skill.name)}
        {@const currPlayer = dpsSkillBreakdownWindow.currPlayer[0]}
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
                  {#if SETTINGS.live.general.state.shortenDps}
                    <AbbreviatedNumber num={skill.totalDmg} />
                  {:else}
                    {skill.totalDmg.toLocaleString()}
                  {/if}
                {:else if col.key === 'dps'}
                  {#if SETTINGS.live.general.state.shortenDps}
                    <AbbreviatedNumber num={skill.dps} />
                  {:else}
                    {Math.round(skill.dps).toLocaleString()}
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
            <TableRowGlow className={className} percentage={SETTINGS.live.general.state.relativeToTopDPSSkill ? maxSkillValue > 0 ? (skill.totalDmg / maxSkillValue) * 100 : 0 : skill.dmgPct} />
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>