<script lang="ts">
  /**
   * @file This component is a button that closes a dialog.
   */
  import { Dialog as DialogPrimitive } from "bits-ui";

  let { ref = $bindable(null), ...restProps }: DialogPrimitive.CloseProps = $props();
</script>

<DialogPrimitive.Close bind:ref data-slot="dialog-close" {...restProps} />
