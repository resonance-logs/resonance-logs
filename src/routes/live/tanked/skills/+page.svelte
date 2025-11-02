<script lang="ts">
  import { commands } from "$lib/bindings";
  import { getClassIcon } from "$lib/utils.svelte";
  import { settings } from "$lib/settings-store";
  import type { SkillsWindow, SkillsUpdatePayload } from "$lib/api";
  import { onTankedSkillsUpdate } from "$lib/api";
  import { getTankedPlayers } from "$lib/stores/live-meter-store.svelte";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import TableRowGlow from "$lib/components/table-row-glow.svelte";
  import { historyDpsSkillColumns } from "$lib/history-columns"; // Use same structure as DPS for consistency
  import AbbreviatedNumber from "$lib/components/abbreviated-number.svelte";
  import PercentFormat from "$lib/components/percent-format.svelte";

  let skillsWindow: SkillsWindow | null = $state(null);
  let unlisten: (() => void) | null = null;

  // Get the playerUid from the URL query parameters
  const playerUid = Number($page.url.searchParams.get("playerUid"));

  // Optimize derived calculations to avoid recalculation on every render
  let maxTakenSkill = $state(0);
  let SETTINGS_YOUR_NAME = $state(settings.state["general"]["showYourName"]);
  let SETTINGS_OTHERS_NAME = $state(settings.state["general"]["showOthersName"]);

  // Update maxTakenSkill when data changes
  $effect(() => {
    maxTakenSkill = skillsWindow?.skillRows.reduce((max, s) => (s.totalDmg > max ? s.totalDmg : max), 0) ?? 0;
  });

  // Update settings references when settings change
  $effect(() => {
    SETTINGS_YOUR_NAME = settings.state["general"]["showYourName"];
    SETTINGS_OTHERS_NAME = settings.state["general"]["showOthersName"];
  });

  // Get visible columns based on settings - use same structure as DPS but for tanked data
  let visibleSkillColumns = $derived.by(() => {
    return historyDpsSkillColumns.filter(col => settings.state.live.tanked.skills[col.key]);
  });

  onMount(() => {
    if (playerUid) {
      // Fetch initial skills data
      commands.getPlayerSkills(playerUid, "tanked").then(result => {
        if (result.status === "ok") {
          skillsWindow = result.data;
        }
      }).catch(e => {
        console.error("Failed to load tanked skills:", e);
      });

      // Listen for updates
      onTankedSkillsUpdate((event: { payload: SkillsUpdatePayload }) => {
        if (event.payload.playerUid === playerUid) {
          skillsWindow = event.payload.skillsWindow;
        }
      }).then(fn => {
        unlisten = fn;
      });
    }

    return () => {
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
    class="sticky top-0 z-10 flex h-8 w-full items-center gap-2 bg-neutral-900 px-2 text-xs"
    style="background-color: {`color-mix(in srgb, ${className ? `var(--class-color-${className.toLowerCase().replace(/\s+/g, '-')})` : '#6b7280'} 30%, transparent)`};"
  >
    <button class="underline" onclick={() => goto("/live/tanked")}>Back</button>
    <span class="font-bold">{currentPlayer.name}</span>
    <span>{currentPlayer.classSpecName}</span>
    <span class="ml-auto">
      <span class="text-xs">Total: </span>
      <AbbreviatedNumber num={currentPlayer.totalDmg} />
    </span>
  </div>
{/if}

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
      {#each skillsWindow?.skillRows as skill (skill.name)}
        {@const className = currentPlayer?.name.includes("You") ? (SETTINGS_YOUR_NAME !== "Hide Your Name" ? currentPlayer.className : "") : SETTINGS_OTHERS_NAME !== "Hide Others' Name" && currentPlayer ? currentPlayer.className : ""}
        <tr 
          class="relative border-t border-neutral-800 hover:bg-neutral-800 transition-colors h-6 text-xs" 
        >
          <td class="px-2 py-1 text-xs text-neutral-300 relative z-10">
            <div class="flex items-center gap-1 h-full">
              <img
                class="size-4 object-contain"
                src={getClassIcon(className)}
                alt="Class icon"
              />
              <span class="truncate">{skill.name}</span>
            </div>
          </td>
          {#each visibleSkillColumns as col (col.key)}
            <td class="px-2 py-1 text-right text-xs text-neutral-300 relative z-10">
              {#if col.key === 'totalDmg'}
                <AbbreviatedNumber num={skill.totalDmg} />
              {:else if col.key === 'dmgPct'}
                <PercentFormat val={skill.dmgPct} fractionDigits={0} />
              {:else if col.key === 'critRate' || col.key === 'critDmgRate' || col.key === 'luckyRate' || col.key === 'luckyDmgRate'}
                <PercentFormat val={skill[col.key]} />
              {:else}
                {col.format(skill[col.key] ?? 0)}
              {/if}
            </td>
          {/each}
          <TableRowGlow className={className} percentage={settings.state.general.relativeToTopDPSPlayer ? maxTakenSkill > 0 ? (skill.totalDmg / maxTakenSkill) * 100 : 0 : skill.dmgPct} />
        </tr>
      {/each}
    </tbody>
  </table>
</div>