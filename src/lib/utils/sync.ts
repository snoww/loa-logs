import { invoke } from "@tauri-apps/api";
import type { Encounter } from "$lib/types";
import pako from "pako";

export const LOG_SITE_URL = "https://logs.snow.xyz";
export const API_URL = "https://api.snow.xyz";
// export const API_URL = "http://localhost:5180";

export const bosses = [
    "Dark Mountain Predator",
    "Ravaged Tyrant of Beasts",
    "Incubus Morphe",
    "Nightmarish Morphe",
    "Covetous Devourer Vykas",
    "Covetous Legion Commander Vykas",
    "Saydon",
    "Kakul",
    "Encore-Desiring Kakul-Saydon",
    "Brelshaza, Monarch of Nightmares",
    "Phantom Legion Commander Brelshaza",
    "Kaltaya, the Blooming Chaos",
    "Rakathus, the Lurking Arrogance",
    "Firehorn, Trampler of Earth",
    "Lazaram, the Trailblazer",
    "Gargadeth",
    "Hanumatan",
    "Caliligos",
    "Achates",
    "Veskal",
    "Argeos",
    "Killineza the Dark Worshipper",
    "Valinak, Herald of the End",
    "Thaemine the Lightqueller",
    "Thaemine, Conqueror of Stars",
    "Red Doom Narkiel",
    "Covetous Master Echidna",
    "Behemoth, the Storm Commander",
    "Behemoth, Cruel Storm Slayer"
];

export async function uploadLog(id: string | number, encounter: Encounter, settings: any) {
    if (!bosses.includes(encounter.currentBossName) || !encounter.cleared) {
        return { id: 0, error: "Invalid Encounter" };
    }

    if (!encounter.difficulty) {
        return { id: 0, error: "Missing Difficulty" };
    }

    const jsonString = JSON.stringify(encounter);
    const compressedData = pako.gzip(jsonString);
    const blob = new Blob([compressedData], { type: "application/octet-stream" });

    const resp = await fetch(API_URL + "/logs/upload", {
        method: "POST",
        headers: {
            access_token: settings.accessToken,
            "Content-Encoding": "gzip",
            "Content-Type": "application/json",
            "visibility": settings.visibility ?? ""
        },
        body: blob
    });

    if (!resp.ok && (resp.status === 500 || resp.status === 401)) {
        let error = "";
        if (resp.status == 500) {
            error = "server bwonk";
        } else if (resp.status == 401) {
            error = "invalid access token";
        }

        await invoke("write_log", {
            message: "couldn't upload encounter " + id + " (" + encounter.currentBossName + ") - error: " + error
        });
        return { id: 0, error: error };
    }
    const body = await resp.json();
    if (body.error) {
        if (body.error === "duplicate log" && body.duplicate) {
            const duplicate = body.duplicate;
            await invoke("write_log", {
                message: "did not upload duplicate encounter " + id + " (" + encounter.currentBossName + ") using existing upstream: " + duplicate
            });
            await invoke("sync", { encounter: Number(id), upstream: duplicate.toString(), failed: false });
            return { id: duplicate, error: "" };
        }

        await invoke("write_log", {
            message:
                "couldn't upload encounter " +
                id +
                " (" +
                encounter.currentBossName +
                ") - error: " +
                body.error.toLowerCase()
        });
        await invoke("sync", { encounter: Number(id), upstream: "0", failed: true });
        return { id: 0, error: body.error };
    }
    if (resp.status === 400) {
        await invoke("write_log", {
            message: "couldn't upload encounter " + id + " (" + encounter.currentBossName + ") - error: unknown error"
        });
        return { id: 0, error: body.error };
    }

    let upstream = body.id;
    if (body.duplicate) {
        upstream = upstream + "-" + body.duplicate;
    }

    await invoke("write_log", {
        message: "uploaded encounter " + id + " (" + encounter.currentBossName + ") upstream: " + upstream
    });
    await invoke("sync", { encounter: Number(id), upstream: upstream.toString(), failed: false });
    return { id: upstream, error: "" };
}

export async function checkAccessToken(accessToken: string) {
    if (!accessToken) {
        return false;
    }

    const resp = await fetch(API_URL + "/logs/token", {
        method: "GET",
        headers: {
            access_token: accessToken
        }
    });

    return resp.ok;
}
