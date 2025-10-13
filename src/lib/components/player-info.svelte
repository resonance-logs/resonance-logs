<script lang="ts">
  import { settings } from "$lib/settings-store";
  import { copyToClipboard, getClassIcon, tooltip } from "$lib/utils.svelte";
  import AbbreviatedNumber from "./abbreviated-number.svelte";

  let {
    className = "",
    classSpecName = "",
    abilityScore = 0,
    name = "",
    uid = 0,
  }: {
    className: string;
    classSpecName: string;
    abilityScore: number;
    name: string;
    uid: number;
  } = $props();

  let SETTINGS_YOUR_NAME = $derived(settings.state.general.showYourName);
  let SETTINGS_OTHERS_NAME = $derived(settings.state.general.showOthersName);

  // Derived helpers
  const isYou = $derived(name?.includes("You") ?? false);
  const classDisplay = $derived(`${className}${classSpecName ? "-" : ""}${classSpecName}`);

  const nameDisplay = $derived(() => {
    const base = name ? name : "Unknown Name";
    if (isYou) {
      if (SETTINGS_YOUR_NAME === "Show Your Class") {
        return `${classDisplay} (You)`;
      } else if (SETTINGS_YOUR_NAME === "Hide Your Name") {
        return "Hidden Name (You)";
      }
      return base;
    } else {
      if (SETTINGS_OTHERS_NAME === "Show Others' Class") {
        return classDisplay;
      } else if (SETTINGS_OTHERS_NAME === "Hide Others' Name") {
        return "Hidden Name";
      }
      return base;
    }
  });

  const classIconDisplay = $derived(() => {
    if (isYou) {
      if (SETTINGS_YOUR_NAME === "Hide Your Name") {
        return "blank";
      }
    } else {
      if (SETTINGS_OTHERS_NAME === "Hide Others' Name") {
        return "blank";
      }
    }
    return className;
  });
</script>

<div class="ml-2 flex">
  <img {@attach tooltip(() => classDisplay)} class="size-5 object-contain" src={getClassIcon(classIconDisplay())} />

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <span class="ml-1 cursor-pointer truncate" onclick={(error) => copyToClipboard(error, `#${uid}`)} {@attach tooltip(() => `UID: #${uid}`)}>
    {#if abilityScore !== 0}
      {#if isYou && settings.state.general.showYourAbilityScore}
        <AbbreviatedNumber num={abilityScore} />
      {:else if !isYou && settings.state.general.showOthersAbilityScore}
        <AbbreviatedNumber num={abilityScore} />
      {/if}
    {:else}
      ??
    {/if}
    {nameDisplay()}
  </span>
</div>
