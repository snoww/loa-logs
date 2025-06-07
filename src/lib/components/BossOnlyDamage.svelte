<script lang="ts">
  import QuickTooltip from "./QuickTooltip.svelte";

  interface Props {
    width?: number;
  }

  let { width = 1.5 }: Props = $props();
</script>

<QuickTooltip tooltip="Boss Only Damage">
  <img src={"/images/icons/boss.png"} alt="Boss Only Damage" style="width: {width}rem;" />
</QuickTooltip>
