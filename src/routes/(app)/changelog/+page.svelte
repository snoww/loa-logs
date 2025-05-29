<script lang="ts">
  import ChangelogMarkdown from "$lib/data/changelog.md?raw";
  import { markdownIt } from "$lib/stores.svelte";
  import Header from "../Header.svelte";
</script>

<Header title="Changelog" />
<div
  class="prose-a:text-accent-500 prose prose-sm prose-zinc prose-invert prose-strong:text-neutrual-200 prose-em:text-neutrual-200 prose-headings:text-neutral-200 prose-img:-my-2 prose-img:rounded-md prose-img:border prose-img:border-neutral-700 prose-img:shadow-md px-14 py-6 text-neutral-200"
>
  {@html $markdownIt.render(ChangelogMarkdown)}
</div>
