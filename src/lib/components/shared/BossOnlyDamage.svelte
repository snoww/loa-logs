<script lang="ts">
    import { imagePath } from "$lib/utils/settings";
    import { getImagePath } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";

    let { width = 1.5 }: { width?: number } = $props();
</script>

<img
    use:tooltip={{ content: "Boss Only Damage" }}
    src={$imagePath.path + getImagePath("icons/boss.png")}
    alt="Boss Only Damage"
    style="width: {width}rem;" />
