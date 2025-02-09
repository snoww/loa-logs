<script lang="ts">
    import { keyboardKeys, registerShortcuts, settings } from "$lib/utils/settings";

    $effect.pre(() => {
        (async () => {
            registerShortcuts($settings.shortcuts);
        })();
    });
</script>

<div class="flex flex-col space-y-4 divide-y-[1px]">
    <div class="mt-4 flex flex-col space-y-2 px-2">
        <div class="flex justify-between">
            <label class="flex items-center" for="modifiers">
                <div class="">
                    <div class="text-gray-100">Show/Hide Meter</div>
                </div>
            </label>
            <div class="flex items-center space-x-2">
                <select
                    id="modifiers"
                    bind:value={$settings.shortcuts.hideMeter.modifier}
                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    <option value="Ctrl">Ctrl</option>
                    <option value="Alt">Alt</option>
                    <option value="Shift">Shift</option>
                </select>
                <div>+</div>
                <select
                    id="keys"
                    bind:value={$settings.shortcuts.hideMeter.key}
                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    {#each keyboardKeys as key}
                        <option value={key}>{key.toUpperCase()}</option>
                    {/each}
                </select>
            </div>
        </div>
        <div class="flex justify-between">
            <label class="flex items-center" for="modifiers">
                <div class="">
                    <div class="text-gray-100">Show/Hide Logs</div>
                </div>
            </label>
            <div class="flex items-center space-x-2">
                <select
                    id="modifiers"
                    bind:value={$settings.shortcuts.showLogs.modifier}
                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    <option value="Ctrl">Ctrl</option>
                    <option value="Alt">Alt</option>
                    <option value="Shift">Shift</option>
                </select>
                <div>+</div>
                <select
                    id="keys"
                    bind:value={$settings.shortcuts.showLogs.key}
                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    {#each keyboardKeys as key}
                        <option value={key}>{key.toUpperCase()}</option>
                    {/each}
                </select>
            </div>
        </div>
        <div class="flex justify-between">
            <label class="flex items-center" for="modifiers">
                <div class="">
                    <div class="text-gray-100">Show Most Recent Encounter</div>
                </div>
            </label>
            <div class="flex items-center space-x-2">
                <select
                    id="modifiers"
                    bind:value={$settings.shortcuts.showLatestEncounter.modifier}
                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    <option value="Ctrl">Ctrl</option>
                    <option value="Alt">Alt</option>
                    <option value="Shift">Shift</option>
                </select>
                <div>+</div>
                <select
                    id="keys"
                    bind:value={$settings.shortcuts.showLatestEncounter.key}
                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    {#each keyboardKeys as key}
                        <option value={key}>{key.toUpperCase()}</option>
                    {/each}
                </select>
            </div>
        </div>
        <div class="flex justify-between">
            <label class="flex items-center" for="modifiers">
                <div class="">
                    <div class="text-gray-100">Reset Session</div>
                </div>
            </label>
            <div class="flex items-center space-x-2">
                <select
                    id="modifiers"
                    bind:value={$settings.shortcuts.resetSession.modifier}
                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    <option value="Ctrl">Ctrl</option>
                    <option value="Alt">Alt</option>
                    <option value="Shift">Shift</option>
                </select>
                <div>+</div>
                <select
                    id="keys"
                    bind:value={$settings.shortcuts.resetSession.key}
                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    {#each keyboardKeys as key}
                        <option value={key}>{key.toUpperCase()}</option>
                    {/each}
                </select>
            </div>
        </div>
        <div class="flex justify-between">
            <label class="flex items-center" for="modifiers">
                <div class="">
                    <div class="text-gray-100">Toggle Pause</div>
                </div>
            </label>
            <div class="flex items-center space-x-2">
                <select
                    id="modifiers"
                    bind:value={$settings.shortcuts.pauseSession.modifier}
                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    <option value="Ctrl">Ctrl</option>
                    <option value="Alt">Alt</option>
                    <option value="Shift">Shift</option>
                </select>
                <div>+</div>
                <select
                    id="keys"
                    bind:value={$settings.shortcuts.pauseSession.key}
                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    {#each keyboardKeys as key}
                        <option value={key}>{key.toUpperCase()}</option>
                    {/each}
                </select>
            </div>
        </div>
        <div class="flex justify-between">
            <label class="flex items-center" for="modifiers">
                <div class="">
                    <div class="text-gray-100">Manual Save</div>
                </div>
            </label>
            <div class="flex items-center space-x-2">
                <select
                    id="modifiers"
                    bind:value={$settings.shortcuts.manualSave.modifier}
                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    <option value="Ctrl">Ctrl</option>
                    <option value="Alt">Alt</option>
                    <option value="Shift">Shift</option>
                </select>
                <div>+</div>
                <select
                    id="keys"
                    bind:value={$settings.shortcuts.manualSave.key}
                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    {#each keyboardKeys as key}
                        <option value={key}>{key.toUpperCase()}</option>
                    {/each}
                </select>
            </div>
        </div>
        <div class="flex justify-between">
            <label class="flex items-center" for="modifiers">
                <div class="">
                    <div class="text-gray-100">Toggle Clickthrough</div>
                </div>
            </label>
            <div class="flex items-center space-x-2">
                <select
                    id="modifiers"
                    bind:value={$settings.shortcuts.disableClickthrough.modifier}
                    class="focus:ring-accent-500 focus:border-accent-500 block w-20 rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    <option value="Ctrl">Ctrl</option>
                    <option value="Alt">Alt</option>
                    <option value="Shift">Shift</option>
                </select>
                <div>+</div>
                <select
                    id="keys"
                    bind:value={$settings.shortcuts.disableClickthrough.key}
                    class="focus:ring-accent-500 focus:border-accent-500 block rounded-lg border border-gray-600 bg-gray-700 p-2.5 text-sm text-white placeholder-gray-400">
                    {#each keyboardKeys as key}
                        <option value={key}>{key.toUpperCase()}</option>
                    {/each}
                </select>
            </div>
        </div>
    </div>
</div>
