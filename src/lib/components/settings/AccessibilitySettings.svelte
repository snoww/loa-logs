<script lang="ts">
    import { settings } from "$lib/utils/settings";
    import { invoke } from "@tauri-apps/api";
    import SettingItem from "./SettingItem.svelte";

    async function toggleBlur() {
        if ($settings.general.blur) {
            await invoke("enable_blur");
        } else {
            await invoke("disable_blur");
        }
    }

    async function toggleBlurWin11() {
        if ($settings.general.blurWin11) {
            await invoke("enable_blur");
        } else {
            await invoke("disable_blur");
        }
    }

</script>

<div class="flex flex-col space-y-4 divide-y-[1px]">
    <div class="mt-4 flex flex-col space-y-2 px-2">
        <div class="flex items-center space-x-2 py-1">
            <div>
                <select
                    id="modifiers"
                    bind:value={$settings.general.scale}
                    class="focus:ring-accent-500 focus:border-accent-500 yx-2 block w-28 rounded-lg border border-gray-600 bg-gray-700 py-1 text-sm text-white placeholder-gray-400">
                    <option value="0">Small</option>
                    <option value="1">Normal</option>
                    <option value="2">Large</option>
                    <option value="3">Largest</option>
                </select>
            </div>
            <div>Meter UI Scale</div>
        </div>
        <div class="flex items-center space-x-2 py-1">
            <div>
                <select
                    id="modifiers"
                    bind:value={$settings.general.logScale}
                    class="focus:ring-accent-500 focus:border-accent-500 yx-2 block w-28 rounded-lg border border-gray-600 bg-gray-700 py-1 text-sm text-white placeholder-gray-400">
                    <option value="0">Small</option>
                    <option value="1">Normal</option>
                    <option value="2">Large</option>
                    <option value="3">Largest</option>
                </select>
            </div>
            <div>Log UI Scale</div>
        </div>
        <SettingItem
            name="Split Lines"
            description={"Split breakdown lines with alternating background colors for better readability."}
            bind:setting={$settings.general.splitLines} />
        <SettingItem
            name="Underline Hovered"
            description="Underlines the text in the row when hovering over it for better readability."
            bind:setting={$settings.general.underlineHovered} />
        <div class="">
            {#if $settings.general.isWin11}
                <label class="flex items-center">
                    <input
                        type="checkbox"
                        bind:checked={$settings.general.blurWin11}
                        on:change={toggleBlurWin11}
                        class="text-accent-500 size-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                    <div class="ml-5">
                        <div class="text-gray-100">Blur Meter Background</div>
                        <div class="text-xs text-gray-300">
                            Adds background blur effect to live meter. On Windows 11, this setting will cause lag while
                            dragging the meter window.
                        </div>
                    </div>
                </label>
            {:else}
                <label class="flex items-center">
                    <input
                        type="checkbox"
                        bind:checked={$settings.general.blur}
                        on:change={toggleBlur}
                        class="text-accent-500 size-5 rounded bg-zinc-700 focus:ring-0 focus:ring-offset-0" />
                    <div class="ml-5">
                        <div class="text-gray-100">Blur Meter Background</div>
                        <div class="text-xs text-gray-300">
                            Adds background blur effect to live meter.
                        </div>
                    </div>
                </label>
            {/if}
        </div>
        {#if $settings.general.isWin11}
            <SettingItem
                name="Transparent Meter"
                description="Turn off to enable Dark Mode for Windows 11 (with blur setting off)."
                bind:setting={$settings.general.transparent} />
        {:else}
            <SettingItem
                name="Transparent Meter"
                description="Toggle transparent background for live meter."
                bind:setting={$settings.general.transparent} />
        {/if}
    </div>
</div>
