<script lang="ts">
  import type { DPSRow, DPSWindow, SkillRow } from "$lib/bindings";
  import { pct } from "$lib/utils";
  // Runes mode props
  const {
    selectedPlayerAndSkills = null,
    dpsWindowPayload,
    clearSelection
  } = $props<{
    selectedPlayerAndSkills: [DPSRow, SkillRow[]] | null;
    dpsWindowPayload: DPSWindow;
    clearSelection: (event: MouseEvent) => void;
  }>();

  // derived safety values (runes mode)
  const playerRow: DPSRow = $derived(selectedPlayerAndSkills?.[0]);
  const skills: SkillRow[] = $derived(selectedPlayerAndSkills?.[1] || []);
  const playerTotal: bigint = $derived(playerRow ? playerRow.totalDmg : 0n);
  const windowTotal: bigint = $derived(dpsWindowPayload?.totalDmg || 0n);
</script>

{#if playerRow}
  <table
    class="table-auto border-collapse border border-gray-300 w-full mt-4"
    oncontextmenu={(e) => {
      e.preventDefault();
      clearSelection(e);
    }}
  >
    <thead>
      <tr class="bg-gray-200">
        <th class="border border-gray-300 px-2 py-1">#</th>
        <th class="border border-gray-300 px-2 py-1">Skill Name</th>
        <th class="border border-gray-300 px-2 py-1">DMG</th>
        <th class="border border-gray-300 px-2 py-1">DPS</th>
        <th class="border border-gray-300 px-2 py-1">D%</th>
        <th class="border border-gray-300 px-2 py-1">CR%</th>
        <th class="border border-gray-300 px-2 py-1">CDmg%</th>
        <th class="border border-gray-300 px-2 py-1">LR%</th>
        <th class="border border-gray-300 px-2 py-1">LDmg%</th>
        <th class="border border-gray-300 px-2 py-1">Hits</th>
        <th class="border border-gray-300 px-2 py-1">Hits/s</th>
      </tr>
    </thead>
    <tbody>
      <tr class="bg-gray-300">
        <td class="border border-gray-300 px-2 py-1 text-center" colspan={2}>{playerRow.name || "??"}</td>
        <td class="border border-gray-300 px-2 py-1">{playerRow.totalDmg}</td>
        <td class="border border-gray-300 px-2 py-1">{playerRow.dps.toFixed()}</td>
        <td class="border border-gray-300 px-2 py-1">{pct(playerRow.totalDmg, windowTotal)}</td>
        <td class="border border-gray-300 px-2 py-1">{playerRow.critRate.toFixed(2)}</td>
        <td class="border border-gray-300 px-2 py-1">{playerRow.critDmgRate.toFixed(2)}</td>
        <td class="border border-gray-300 px-2 py-1">{playerRow.luckyRate.toFixed(2)}</td>
        <td class="border border-gray-300 px-2 py-1">{playerRow.luckyDmgRate.toFixed(2)}</td>
        <td class="border border-gray-300 px-2 py-1">{playerRow.hits}</td>
        <td class="border border-gray-300 px-2 py-1">{playerRow.hitsPerSecond.toFixed(2)}</td>
      </tr>
      {#each skills as skill, index}
        <tr class="bg-white">
          <td class="border border-gray-300 px-2 py-1">{index + 1}</td>
          <td class="border border-gray-300 px-2 py-1">{@html skill.name}</td>
          <td class="border border-gray-300 px-2 py-1">{skill.totalDmg}</td>
          <td class="border border-gray-300 px-2 py-1">{skill.dps.toFixed()}</td>
          <td class="border border-gray-300 px-2 py-1">{pct(skill.totalDmg, playerTotal)}</td>
          <td class="border border-gray-300 px-2 py-1">{skill.critRate.toFixed(2)}</td>
          <td class="border border-gray-300 px-2 py-1">{skill.critDmgRate.toFixed(2)}</td>
          <td class="border border-gray-300 px-2 py-1">{skill.luckyRate.toFixed(2)}</td>
          <td class="border border-gray-300 px-2 py-1">{skill.luckyDmgRate.toFixed(2)}</td>
          <td class="border border-gray-300 px-2 py-1">{skill.hits}</td>
          <td class="border border-gray-300 px-2 py-1">{skill.hitsPerSecond.toFixed(2)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{:else}
  <!-- No player selected / data not ready -->
{/if}
