<script lang="ts">
    import { imagePath } from "$lib/utils/settings";
    import { getImagePath } from "$lib/utils/strings";
    import { tooltip } from "$lib/utils/tooltip";

    export let width = 1.5;
</script>

<img
    use:tooltip={{ content: "Boss Only Damage" }}
    src={$imagePath.path + getImagePath("icons/boss.png")}
    alt="Boss Only Damage"
    style="width: {width}rem;" />
