<script lang="ts" module>
  export type ToastData = {
    title: string;
    description: string;
    color: string;
  };

  const {
    elements,
    helpers,
    states: { toasts },
    actions: { portal }
  } = createToaster<ToastData>();

  export const addToast = helpers.addToast;
</script>

<script lang="ts">
  import { createToaster } from "@melt-ui/svelte";
  import { flip } from "svelte/animate";
  import Toast from "./Toast.svelte";
  import { settings } from "$lib/stores.svelte";
  import { page } from "$app/state";

  const live = $derived(page.route.id === "/(live)/live");
</script>

<div
  class="fixed top-auto z-50 m-4 flex flex-col {settings.app.general.accentColor} {live
    ? 'inset-x-0 bottom-3 items-center gap-1'
    : 'bottom-0 right-0 items-end gap-2'} 
  "
  use:portal
>
  {#each $toasts as toast (toast.id)}
    <div animate:flip={{ duration: 500 }}>
      <Toast {elements} {toast} />
    </div>
  {/each}
</div>
