<script lang="ts">
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import type { Snippet } from "svelte";

  import Header from "../Header.svelte";

  const { children }: { children?: Snippet } = $props();

  let pathname = $derived(page.url.pathname);

  function isActive(path: string) {
    return pathname === path || pathname.startsWith(`${path}/`);
  }
</script>

<div class="flex h-screen flex-col overflow-hidden">
  <Header title="Stats"></Header>

  <div class="border-b border-neutral-800 bg-neutral-900/80">
    <nav class="mx-auto flex h-12 max-w-[180rem] items-center gap-1 px-6" aria-label="Statistics sections">
      <a
        href={resolve("/statistics/characters")}
        class="rounded-md px-3 py-1.5 text-sm text-neutral-400 hover:bg-neutral-800/80 hover:text-neutral-100"
        class:bg-neutral-800={isActive("/statistics/characters")}
        class:text-neutral-100={isActive("/statistics/characters")}
      >
        Character Stats
      </a>
      <a
        href={resolve("/statistics/progression")}
        class="rounded-md px-3 py-1.5 text-sm text-neutral-400 hover:bg-neutral-800/80 hover:text-neutral-100"
        class:bg-neutral-800={isActive("/statistics/progression")}
        class:text-neutral-100={isActive("/statistics/progression")}
      >
        Raid Progression
      </a>
    </nav>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto">
    {@render children?.()}
  </div>
</div>
