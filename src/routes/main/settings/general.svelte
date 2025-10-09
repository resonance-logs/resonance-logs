<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { settings } from "$lib/settings-store";
  import SettingsSelect from "./settings-select.svelte";
  import SettingsSwitch from "./settings-switch.svelte";

  const SETTINGS_CATEGORY = "general";
  const GENERAL_SETTINGS = $derived(settings.state[SETTINGS_CATEGORY]);

  // TODO: Add a typing inheritance system to automatically generate these
</script>

<Tabs.Content value={SETTINGS_CATEGORY}>
  <SettingsSelect bind:selected={GENERAL_SETTINGS["showYourName"]} values={["Show Your Name", "Show Your Class", "Hide Your Name"]} label="Show Your Name" description="Show Your Class = replace your name with your class" />
  <SettingsSelect bind:selected={GENERAL_SETTINGS["showOthersName"]} values={["Show Others' Name", "Show Others' Class", "Hide Others' Name"]} label="Show Others' Name" description="Show Others' Class = replace others' name with their class" />
  <SettingsSwitch bind:checked={GENERAL_SETTINGS["showYourAbilityScore"]} label="Your Ability Score" description="Show your ability score" />
  <SettingsSwitch bind:checked={GENERAL_SETTINGS["showOthersAbilityScore"]} label="Others' Ability Score" description="Show others' ability score" />
</Tabs.Content>
