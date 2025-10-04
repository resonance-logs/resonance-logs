<script lang="ts">
  import { copyToClipboard, getClassIcon, tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "./abbreviated-number.svelte";

  let {
    className = "",
    classSpecName = "",
    abilityScore = 0n,
    name = "",
    uid = 0n,
  }: {
    className: string;
    classSpecName: string;
    abilityScore: bigint;
    name: string;
    uid: bigint;
  } = $props();
</script>

<div class="ml-2 flex">
  <img {@attach tooltip(() => `${className}${classSpecName ? "-" : ""}${classSpecName}`)} class="size-5" src={getClassIcon(className)} alt={`${className} class icon`} />
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <span class="cursor-pointer truncate" onclick={(error) => copyToClipboard(error, `#${uid}`)} {@attach tooltip(() => `UID: #${uid}`)}>
      {#if abilityScore !== 0n}
        <AbbreviatedNumber num={abilityScore} />
      {:else}
        ??
      {/if}
      {name ?? "Unknown Name"}
    </span>
</div>
