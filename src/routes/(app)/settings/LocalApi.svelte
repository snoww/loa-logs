<script lang="ts">
  import { getLocalApiStatus, restartLocalApi, saveSettings, type LocalApiStatus } from "$lib/api";
  import { addToast } from "$lib/components/Toaster.svelte";
  import { settings } from "$lib/stores.svelte";
  import { onMount } from "svelte";

  let showToken = $state(false);
  let status = $state<LocalApiStatus | null>(null);
  let checking = $state(false);
  let applying = $state(false);
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

  function normalizeSettings() {
    syncOrigins();
    if (!settings.app.localApi.port || settings.app.localApi.port < 1) {
      settings.app.localApi.port = 16724;
    }
    if (settings.app.localApi.enabled && !settings.app.localApi.token) {
      settings.app.localApi.token = genToken();
      showToken = true;
    }
  }

  async function apply(expectRunning = settings.app.localApi.enabled) {
    applying = true;
    try {
      normalizeSettings();
      await saveSettings(settings.app);
      await restartLocalApi();
      const next = await pollStatus(expectRunning);

      if (!settings.app.localApi.enabled) {
        addToast({ data: { title: "", description: "Local API stopped", color: "border-neutral-500/30" } });
      } else if (next?.running) {
        addToast({
          data: {
            title: "",
            description: `Local API running on 127.0.0.1:${next.port}`,
            color: "border-green-500/30"
          }
        });
      } else {
        addToast({
          data: {
            title: "",
            description: next?.error ? `Local API error: ${next.error}` : "Local API did not start",
            color: "border-red-500/30"
          }
        });
      }
    } catch (e) {
      addToast({
        data: {
          title: "",
          description: e instanceof Error ? e.message : "Local API settings failed",
          color: "border-red-500/30"
        }
      });
    } finally {
      applying = false;
    }
  }

  async function setEnabled(enabled: boolean) {
    settings.app.localApi.enabled = enabled;
    await apply(enabled);
  }

  async function pollStatus(expectRunning = true): Promise<LocalApiStatus | null> {
    checking = true;
    try {
      for (let i = 0; i < 10; i++) {
        status = await getLocalApiStatus();
        if (!expectRunning || status.running || status.error) break;
        await new Promise((r) => setTimeout(r, 300));
      }
      return status;
    } finally {
      checking = false;
    }
  }

  onMount(() => {
    void pollStatus(settings.app.localApi.enabled);
  });
</script>

<div class="flex max-w-2xl flex-col gap-4">
  <div class="text-sm text-neutral-300">
    Exposes a <span class="font-semibold">read-only</span> API on
    <span class="font-mono">127.0.0.1</span> so allow-listed websites can read sanitized clear summaries. Disabled by default.
  </div>

  <label class="flex items-center gap-2">
    <input
      type="checkbox"
      checked={settings.app.localApi.enabled}
      onchange={(e) => setEnabled((e.currentTarget as HTMLInputElement).checked)}
      class="form-checkbox size-5 rounded-sm border-0 bg-neutral-700 checked:text-accent-600/80 focus:ring-0"
    />
    <div class="ml-3">
      <div class="text-sm">Enable Local API</div>
      <div class="text-xs text-neutral-300">Starts or stops the localhost-only HTTP server.</div>
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
      </div>
    </div>

    <div class="flex flex-col gap-1">
      <div class="text-sm">Allowed Origins</div>
      <textarea
        class="h-28 w-96 form-textarea rounded-md border-0 bg-neutral-700 font-mono text-xs focus:ring-0"
        bind:value={originsText}
        onblur={syncOrigins}
        placeholder="https://example.com"
      ></textarea>
      <div class="text-xs text-neutral-300">One origin per line. Only these websites may read your meter data.</div>
    </div>
  {/if}

  <div class="flex items-center gap-3">
    {#if settings.app.localApi.enabled}
      <button
        class="rounded-md bg-accent-800/80 px-3 py-1.5 text-sm hover:bg-accent-700/80 disabled:opacity-50"
        disabled={applying || checking}
        onclick={() => apply(true)}
      >
        {applying ? "Applying…" : "Apply / Restart"}
      </button>
    {/if}
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
