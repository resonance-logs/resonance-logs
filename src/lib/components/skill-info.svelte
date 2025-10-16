<script lang="ts">
  let {
    skillName = "Unknown name",
  }: {
    skillName: string;
  } = $props();
</script>

<span class="ml-2 flex"><span class="truncate">{skillName}</span></span>
