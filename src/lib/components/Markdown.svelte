<script lang="ts" module>
  import { markdownIt } from "$lib/stores.svelte";

  export { markdown };
</script>

{#snippet markdown(body: string)}
  <div
    class="prose-a:text-accent-500 prose prose-sm prose-zinc prose-invert prose-strong:text-neutrual-200 prose-em:text-neutrual-200 prose-headings:text-neutral-200 prose-img:-my-2 prose-img:rounded-md prose-img:border prose-img:border-neutral-700 prose-img:shadow-md mx-auto max-w-[180rem] px-14 py-6 text-neutral-200"
  >
    {@html markdownIt.render(body)}
  </div>
{/snippet}
