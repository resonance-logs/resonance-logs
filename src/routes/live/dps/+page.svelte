<script lang="ts">
  import { onMount } from "svelte";
  import { commands, type DPSWindow } from "$lib/bindings";
  import { abbreviateNumberSplit, getClassColor, getClassIcon, pct } from "$lib/utils";
  import { goto } from "$app/navigation";

  onMount(() => {
    fetchData();
    const interval = setInterval(fetchData, 200);

    return () => clearInterval(interval);
  });

  // is there a better way to init these?
  let dpsWindow: DPSWindow = $state({ dpsRows: [], totalDmg: 0n, elapsedMs: 0n });

  async function fetchData() {
    try {
      dpsWindow = await commands.getDamageRow();
      console.log(+Date.now(), $state.snapshot(dpsWindow));
    } catch (e) {
      console.error("Error fetching data:", e);
    }
  }
</script>

<div class="relative flex flex-col">
  <table class="w-screen table-fixed">
    <thead class="z-1 sticky top-0 h-6">
      <tr class="bg-neutral-900">
        <th class="w-3/8 pl-2 text-left">UID</th>
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
      {#each dpsWindow.dpsRows as dpsRow (dpsRow.uid)}
        <tr
          class="h-7 px-2 py-1 text-center"
          onclick={() => {
            goto(`/live/dps/dpsSkillBreakdown?playerUid=${dpsRow.uid}`);
          }}
        >
          <td class="truncate pl-2 text-left">{dpsRow.uid}</td>
          <td><img class="ml-2 size-5 object-contain" src={getClassIcon(dpsRow.class)} alt={`${dpsRow.class} class icon`} /></td>
          <td>
            <span class="flex">
              <span class="truncate">
                {`${dpsRow.abilityScore && dpsRow.abilityScore !== 0 ? dpsRow.abilityScore : "??"} ${dpsRow.name?.trim() ? dpsRow.name : "Unknown"}`}
              </span>
            </span>
          </td>
          <td>{abbreviateNumberSplit(Number(dpsRow.totalDmg))[0]}<span class="text-tiny text-gray-300">{abbreviateNumberSplit(Number(dpsRow.totalDmg))[1]}</span></td>
          <td>{abbreviateNumberSplit(dpsRow.dps)[0]}<span class="text-tiny text-gray-300">{abbreviateNumberSplit(dpsRow.dps)[1]}</span></td>
          <td>{pct(dpsRow.totalDmg, dpsWindow.totalDmg).toFixed(0)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsRow.critRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsRow.critDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsRow.luckyRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsRow.luckyDmgRate.toFixed(1)}<span class="text-tiny text-gray-300">%</span></td>
          <td>{dpsRow.hits}</td>
          <td>{dpsRow.hitsPerSecond.toFixed(1)}</td>
          <td class="-z-1 absolute left-0 h-7" style="background-color: {getClassColor(dpsRow.class)}; width: {pct(dpsRow.totalDmg, dpsWindow.totalDmg)}vw;"></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
