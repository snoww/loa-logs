<script lang="ts">
    import { tooltip } from "$lib/utils/tooltip";

    export let width = 1.5;
</script>

<img use:tooltip={{ content: "Boss Only Damage" }} src="/boss.png" alt="Boss Only Damage" style="width: {width}rem;" />
