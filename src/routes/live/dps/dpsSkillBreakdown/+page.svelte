<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type DPSSkillBreakdownWindow } from "$lib/bindings";
  import { abbreviateNumberSplit, getClassColor, getClassIcon, pct } from "$lib/utils";
  import { page } from "$app/state";

  const playerUid: string = page.url.searchParams.get("playerUid") ?? "0";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);
    return () => clearInterval(interval);
  });

  let dpsSkillBreakdownWindow: DPSSkillBreakdownWindow = $state({ currPlayer: { uid: 0n, name: "", class: "", abilityScore: 0, totalDmg: 0n, dps: 0, critRate: 0, critDmgRate: 0, luckyRate: 0, luckyDmgRate: 0, hits: 0n, hitsPerSecond: 0 }, skillRows: [], totalDmg: 0n, elapsedMs: 0n });

  async function fetchData() {
    try {
      dpsSkillBreakdownWindow = await commands.getSkillRow(playerUid);
      console.log(+Date.now(), $state.snapshot(dpsSkillBreakdownWindow));
    } catch (e) {
      console.error("Error fetching data:", e);
    }
  }
</script>

<div class="relative flex flex-col">
  <table
    class="w-screen table-fixed"
    oncontextmenu={() => {
      window.history.back();
    }}
  >
    <thead class="z-1 sticky top-0 h-6">
      <tr class="bg-neutral-900">
        <th class="w-5 px-4"></th>
        <!-- Image -->
        <th class="w-5/8"></th>
        <!-- Ability Score + Name -->
        <th class="w-12">DMG</th>
        <th class="w-12">DPS</th>
        <th class="w-12">D<span class="text-tiny text-gray-300">%</span></th>
        <th class="w-12">CR<span class="text-tiny text-gray-300">%</span></th>
        <th class="w-13">CDMG<span class="text-tiny text-gray-300">%</span></th>
        <th class="w-12">LR<span class="text-tiny text-gray-300">%</span></th>
        <th class="w-13">LDMG<span class="text-tiny text-gray-300">%</span></th>
        <th class="w-13">Hits</th>
        <th class="w-13">Hits/s</th>
      </tr>
    </thead>
    <tbody>
      <tr class="h-7 px-2 py-1 text-center">
        <td><img class="ml-2 size-5 object-contain" src={getClassIcon(dpsSkillBreakdownWindow.currPlayer.class)} alt={`${dpsSkillBreakdownWindow.currPlayer.class} class icon`} /></td>
        <td><span class="flex"><span class="truncate">{`${dpsSkillBreakdownWindow.currPlayer.abilityScore && dpsSkillBreakdownWindow.currPlayer.abilityScore !== 0 ? dpsSkillBreakdownWindow.currPlayer.abilityScore : "??"} ${dpsSkillBreakdownWindow.currPlayer.name?.trim() ? dpsSkillBreakdownWindow.currPlayer.name : "Unknown"}`}</span></span> </td>
        <td>{abbreviateNumberSplit(Number(dpsSkillBreakdownWindow.currPlayer.totalDmg))[0]}<span class="text-tiny text-gray-300">{abbreviateNumberSplit(Number(dpsSkillBreakdownWindow.currPlayer.totalDmg))[1]}</span></td>
        <td>{abbreviateNumberSplit(dpsSkillBreakdownWindow.currPlayer.dps)[0]}<span class="text-tiny text-gray-300">{abbreviateNumberSplit(dpsSkillBreakdownWindow.currPlayer.dps)[1]}</span></td>
        <td>{pct(dpsSkillBreakdownWindow.currPlayer.totalDmg, dpsSkillBreakdownWindow.totalDmg).toFixed(0)}<span class="text-tiny text-gray-300">%</span></td>
        <td>{dpsSkillBreakdownWindow.currPlayer.critRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
        <td>{dpsSkillBreakdownWindow.currPlayer.critDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
        <td>{dpsSkillBreakdownWindow.currPlayer.luckyRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
        <td>{dpsSkillBreakdownWindow.currPlayer.luckyDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
        <td>{dpsSkillBreakdownWindow.currPlayer.hits}</td>
        <td>{dpsSkillBreakdownWindow.currPlayer.hitsPerSecond.toFixed(1)}</td>
        <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(dpsSkillBreakdownWindow.currPlayer.class)}; width: 100vw;"></td>
      </tr>
      <!-- TODO: get some sort of key for this each loop -->
      <!-- eslint-disable svelte/require-each-key -->
      {#each dpsSkillBreakdownWindow.skillRows as skillRow, i}
        <tr class="h-7 px-2 py-1 text-center">
          <td><img class="ml-2 size-5 object-contain" src="data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=" alt="blank for now" /></td>
          <td><span class="flex"><span class="truncate">{skillRow.name}</span></span></td>
          <td>{abbreviateNumberSplit(Number(skillRow.totalDmg))[0]}<span class="text-tiny text-gray-300">{abbreviateNumberSplit(Number(skillRow.totalDmg))[1]}</span></td>
          <td>{abbreviateNumberSplit(skillRow.dps)[0]}<span class="text-tiny text-gray-300">{abbreviateNumberSplit(skillRow.dps)[1]}</span></td>
          <td>{pct(skillRow.totalDmg, dpsSkillBreakdownWindow.currPlayer.totalDmg).toFixed(0)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{skillRow.critRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{skillRow.critDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{skillRow.luckyRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{skillRow.luckyDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{skillRow.hits}</td>
          <td>{skillRow.hitsPerSecond.toFixed(1)}</td>
          <td class="-z-1 absolute left-0 h-7" style="background-color: {`color-mix(in srgb, ${getClassColor(dpsSkillBreakdownWindow.currPlayer.class)} 80%, white ${i % 2 === 0 ? '30%' : '10%'})`}; width: {pct(skillRow.totalDmg, dpsSkillBreakdownWindow.currPlayer.totalDmg)}vw;"></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
