<script lang="ts">
  import { getLocalApiStatus, restartLocalApi, saveSettings, type LocalApiStatus } from "$lib/api";
  import { addToast } from "$lib/components/Toaster.svelte";
  import { settings } from "$lib/stores.svelte";
  import { onMount } from "svelte";

  let showToken = $state(false);
  let status = $state<LocalApiStatus | null>(null);
  let checking = $state(false);
  let originsText = $state(settings.app.localApi.allowedOrigins.join("\n"));

  function genToken(): string {
    const bytes = new Uint8Array(32);
    crypto.getRandomValues(bytes);
    return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
  }

  function regenerateToken() {
    settings.app.localApi.token = genToken();
    showToken = true;
  }

  async function copyToken() {
    try {
      await navigator.clipboard.writeText(settings.app.localApi.token);
      addToast({ data: { title: "", description: "Token copied", color: "border-green-500/30" } });
    } catch (e) {
      console.error(e);
    }
  }

  function syncOrigins() {
    settings.app.localApi.allowedOrigins = originsText
      .split("\n")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  }

  async function apply() {
    syncOrigins();
    // Guard against an emptied/invalid port field persisting a null.
    if (!settings.app.localApi.port || settings.app.localApi.port < 1) {
      settings.app.localApi.port = 16724;
    }
    // A token is required for the server to start.
    if (settings.app.localApi.enabled && !settings.app.localApi.token) {
      settings.app.localApi.token = genToken();
    }
    await saveSettings(settings.app);
    // Reconcile only happens here (not on every save), so unrelated saves and
    // multi-window settings sync never restart the listener.
    await restartLocalApi();
    // The server binds asynchronously, so poll until it's actually up (or
    // errors) rather than flashing a stale "stopped".
    await pollStatus(settings.app.localApi.enabled);
    addToast({ data: { title: "", description: "Local API settings applied", color: "border-green-500/30" } });
  }

  async function pollStatus(expectRunning = true) {
    checking = true;
    try {
      for (let i = 0; i < 10; i++) {
        status = await getLocalApiStatus();
        if (!expectRunning || status.running || status.error) break;
        await new Promise((r) => setTimeout(r, 300));
      }
    } finally {
      checking = false;
    }
  }

  async function refreshStatus() {
    checking = true;
    try {
      status = await getLocalApiStatus();
    } catch (e) {
      console.error(e);
      status = null;
    } finally {
      checking = false;
    }
  }

  onMount(refreshStatus);
</script>

<div class="flex max-w-2xl flex-col gap-4">
  <div class="text-sm text-neutral-300">
    Exposes a <span class="font-semibold">read-only</span> API on
    <span class="font-mono">127.0.0.1</span> so an allow-listed website (e.g. neria.dev) can detect your private
    clears. Only sanitized clear summaries are shared — never combat logs, damage, or party details. Disabled by
    default.
  </div>

  <label class="flex items-center gap-2">
    <input
      type="checkbox"
      bind:checked={settings.app.localApi.enabled}
      class="form-checkbox size-5 rounded-sm border-0 bg-neutral-700 checked:text-accent-600/80 focus:ring-0"
    />
    <div class="ml-3">
      <div class="text-sm">Enable Local API</div>
      <div class="text-xs text-neutral-300">Starts a localhost-only HTTP server when applied.</div>
    </div>
  </label>

  {#if settings.app.localApi.enabled}
    <label class="flex items-center">
      <input
        type="number"
        class="form-input h-8 w-24 rounded-md border-0 bg-neutral-700 text-sm focus:ring-0"
        bind:value={settings.app.localApi.port}
      />
      <div class="ml-5">
        <div class="text-sm">Port</div>
        <div class="text-xs text-neutral-300">Default is 16724.</div>
      </div>
    </label>

    <div class="flex flex-col gap-1">
      <div class="text-sm">API Token</div>
      <div class="flex items-center gap-2">
        <input
          type={showToken ? "text" : "password"}
          class="form-input h-8 w-96 rounded-md border-0 bg-neutral-700 font-mono text-xs focus:ring-0"
          bind:value={settings.app.localApi.token}
          placeholder="generate a token"
        />
        <button
          class="rounded-md bg-neutral-700 px-2 py-1 text-xs hover:bg-neutral-600"
          onclick={() => (showToken = !showToken)}
        >
          {showToken ? "Hide" : "Show"}
        </button>
        <button class="rounded-md bg-neutral-700 px-2 py-1 text-xs hover:bg-neutral-600" onclick={copyToken}>
          Copy
        </button>
        <button class="rounded-md bg-neutral-700 px-2 py-1 text-xs hover:bg-neutral-600" onclick={regenerateToken}>
          Regenerate
        </button>
      </div>
      <div class="text-xs text-neutral-300">
        Required for all data requests (sent as <span class="font-mono">Authorization: Bearer &lt;token&gt;</span>).
        Paste this into Neria's local meter settings.
      </div>
    </div>

    <div class="flex flex-col gap-1">
      <div class="text-sm">Allowed Origins</div>
      <textarea
        class="form-textarea h-28 w-96 rounded-md border-0 bg-neutral-700 font-mono text-xs focus:ring-0"
        bind:value={originsText}
        onblur={syncOrigins}
      ></textarea>
      <div class="text-xs text-neutral-300">One origin per line. Only these websites may read your meter data.</div>
    </div>
  {/if}

  <div class="flex items-center gap-3">
    <button class="bg-accent-800/80 hover:bg-accent-700/80 rounded-md px-3 py-1.5 text-sm" onclick={apply}>
      Apply / Restart
    </button>
    <button class="rounded-md bg-neutral-700 px-3 py-1.5 text-sm hover:bg-neutral-600" onclick={refreshStatus}>
      Refresh Status
    </button>
    <div class="text-xs">
      {#if checking}
        <span class="text-neutral-400">checking…</span>
      {:else if status?.running}
        <span class="text-green-400">running on 127.0.0.1:{status.port}</span>
      {:else if status?.error}
        <span class="text-red-400">error: {status.error}</span>
      {:else}
        <span class="text-neutral-400">stopped</span>
      {/if}
    </div>
  </div>
</div>
