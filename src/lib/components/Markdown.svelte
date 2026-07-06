<script lang="ts" module>
  import { markdownIt } from "$lib/stores.svelte";

  export { markdown };
</script>

{#snippet markdown(body: string)}
  <div
    class="prose-strong:text-neutrual-200 prose-em:text-neutrual-200 mx-auto prose prose-sm max-w-[180rem] px-14 py-6 text-neutral-200 prose-zinc prose-invert prose-headings:text-neutral-200 prose-a:text-accent-500 prose-img:-my-2 prose-img:rounded-md prose-img:border prose-img:border-neutral-700 prose-img:shadow-md"
  >
    {@html markdownIt.render(body)}
  </div>
{/snippet}
