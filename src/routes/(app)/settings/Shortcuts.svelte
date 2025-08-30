<script lang="ts">
  import { misc, settings } from "$lib/stores.svelte";
  import { registerShortcuts, shortcuts } from "$lib/utils/shortcuts";
  import { createDialog, melt } from "@melt-ui/svelte";
  import { unregisterAll } from "@tauri-apps/plugin-global-shortcut";
  import { onDestroy, onMount } from "svelte";
  import { fade } from "svelte/transition";

  const {
    elements: { trigger, portalled, overlay, content, title, description, close },
    states: { open }
  } = createDialog();

  let keys: string[] = $state([]);
  let currentAction: string | null = $state(null);

  let keyUp = $state(true);
  const handleKeydown = (e: KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();

    if (keyUp) {
      keys = [];
      keyUp = false;
    }

    // Handle modifier keys
    const modifiers = [];
    if (e.ctrlKey) modifiers.push("Ctrl");
    if (e.altKey) modifiers.push("Alt");
    if (e.shiftKey) modifiers.push("Shift");

    // Get the physical key
    let keyName = e.code;
    if (keyName.startsWith("Key")) {
      keyName = keyName.slice(3); // KeyA -> A
    } else if (keyName.startsWith("Digit")) {
      keyName = keyName.slice(5); // Digit1 -> 1
    } else if (keyName.startsWith("Numpad")) {
      keyName = keyName; // Keep as is for numpad keys
    }
    
    // Don't add modifier keys as separate keys
    if (!["Control", "Alt", "Shift"].includes(e.key)) {
      const fullKey = [...modifiers, keyName].join("+");
      if (keys.indexOf(fullKey) === -1) {
        keys = [fullKey]; // Replace the array with the full combination
      }
    } else if (modifiers.length > 0) {
      // Just show modifiers while they're being pressed
      keys = [modifiers.join("+")];
    }
  };

  const handleKeyUp = () => {
    keyUp = true;
  };

  const listen = () => {
    keys = [];
    document.addEventListener("keydown", handleKeydown);
    document.addEventListener("keyup", handleKeyUp);
  };

  $effect(() => {
    if (!$open) {
      document.removeEventListener("keydown", handleKeydown);
      document.removeEventListener("keyup", handleKeyUp);
    }
  });

  onMount(() => {
    misc.modifyingShortcuts = true;
    (async () => {
      await unregisterAll();
    })();
  });

  onDestroy(() => {
    misc.modifyingShortcuts = false;
    document.removeEventListener("keydown", handleKeydown);
    document.removeEventListener("keyup", handleKeyUp);
    registerShortcuts();
  });
</script>

{#snippet shortcutOption(action: string, shortcut: string)}
  <div class="flex min-w-80 items-center justify-between gap-2 rounded px-2 py-1 hover:bg-neutral-700/60">
    <p>{shortcuts[action].name}</p>
    <button
      class="min-w-40 rounded-md bg-neutral-700 px-2 py-1 font-mono text-xs"
      onclick={() => {
        currentAction = action;
        listen();
      }}
      use:melt={$trigger}
    >
      {shortcut || "None"}
    </button>
  </div>
{/snippet}
<div class="flex flex-col gap-2">
  <div class="w-fit rounded-md bg-red-500/40 px-2 py-1">Shortcuts are disabled until you leave this page</div>
  {#each Object.entries(settings.app.shortcuts) as shortcut}
    {@render shortcutOption(shortcut[0], shortcut[1])}
  {/each}
</div>

{#if $open}
  <div use:melt={$portalled}>
    <div use:melt={$overlay} class="fixed inset-0 z-50 bg-black/50" transition:fade={{ duration: 150 }}></div>
    <div
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw] max-w-[450px] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-neutral-800 p-4 shadow-lg
      {settings.app.general.accentColor} flex flex-col items-center gap-4 text-white"
      use:melt={$content}
    >
      <h2 use:melt={$title} class="font-semibold">Record Shortcut</h2>
      <p use:melt={$description} class="min-w-40 rounded bg-neutral-900 px-4 text-center font-mono">
        {keys.length === 0 ? "listening..." : keys.join(" + ")}
      </p>
      <div class="gap-30 flex items-center pt-5">
        <button
          use:melt={$close}
          class="bg-accent-500/70 hover:bg-accent-500/60 rounded-md px-2 py-1"
          onclick={() => {
            if (currentAction) {
              settings.app.shortcuts[currentAction as keyof typeof settings.app.shortcuts] = keys.join("+");
            }
          }}
        >
          Save
        </button>
      </div>
    </div>
  </div>
{/if}
