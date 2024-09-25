<script lang="ts">
    import LogSidebar from "$lib/components/logs/LogSidebar.svelte";
    import { backNavStore, markdownIt, pageStore, searchStore } from "$lib/utils/stores";
    import { onMount } from "svelte";
    import Title from "$lib/components/shared/Title.svelte";
    import ChangelogMarkdown from "$lib/data/changelog.md?raw";
    import { miscSettings } from "$lib/utils/settings";

    let hidden: boolean = true;

    onMount(() => {
        $pageStore = 1;
        $backNavStore = false;
        $searchStore = "";
        $miscSettings.viewedChangelog = true;
    });
</script>

<LogSidebar bind:hidden />
<div class="custom-scroll h-screen overflow-y-scroll bg-zinc-800 pb-8">
    <div class="sticky top-0 flex h-16 justify-between bg-zinc-800 px-8 py-5 shadow-md">
        <Title text="Changelog" bind:hidden />
    </div>
    <div
        class="prose-a:text-accent-500 prose prose-sm prose-zinc prose-invert mx-14 my-6 tracking-tight text-gray-200 prose-img:-my-2 prose-img:rounded-md prose-img:border prose-img:border-zinc-600 prose-img:shadow-md">
        {@html $markdownIt.render(ChangelogMarkdown)}
    </div>
</div>
