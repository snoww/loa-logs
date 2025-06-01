<script lang="ts">
  import { markdown } from "$lib/components/Markdown.svelte";
  import ChangelogMarkdown from "$lib/data/changelog.md?raw";
  import Header from "../Header.svelte";
</script>

<Header title="Changelog" />
{@render markdown(ChangelogMarkdown)}
