import { writable } from "svelte/store";

export interface AppContext {
    productName: string;
    donateUrl: string;
    discordUrl: string;
    discordName: string;
    version: string;
}

export const appContext = writable<AppContext>({
    productName: "LOA Logs",
    donateUrl: "https://ko-fi.com/synow",
    discordUrl: "https://discord.gg/RXvTMV2YHu",
    discordName: "ramen shop",
    version: ""
});