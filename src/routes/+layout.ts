import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { miscSettings as miscSettingsStore } from "$lib/utils/settings";
import { getVersion } from "@tauri-apps/api/app";
import { get } from "svelte/store";

export const prerender = true;
export const ssr = false;

async function gotoChangelog() {
    if (browser) {
        const { appWindow } = await import("@tauri-apps/api/window");
        await appWindow.show();
        await appWindow.unminimize();
        await appWindow.setFocus();

        goto("/changelog");
    }
}

export async function load({ url: { pathname } }) {
    if (pathname !== "/" && pathname !== "/changelog") {
        const miscSettings = get(miscSettingsStore);

        if (miscSettings) {
            const version = await getVersion();
            if (!miscSettings.viewedChangelog || miscSettings.version !== version) {
                miscSettings.version = version;
                miscSettingsStore.set(miscSettings);
                await gotoChangelog();
            }
        } else {
            miscSettingsStore.set({ version: await getVersion() });
            await gotoChangelog();
        }
    }
}
