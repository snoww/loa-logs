import { writable } from "svelte/store";
import { Peer, type DataConnection } from "peerjs";

// If true, a server is set up to listen for incoming connections.
export const liveConnectionListeningStore = writable(false);
export const liveServerListeningAlert = writable(false);

let peer: Peer | null = null;
let connections: DataConnection[] = [];

export async function broadcastLiveMessage(message: unknown) {
    await Promise.all(connections.map((c) => c.send(message)));
}

export async function stopHosting() {
    if (peer) {
        peer.destroy();
        peer = null;
    }

    connections = [];
    liveConnectionListeningStore.set(false);
}

export async function startHosting(): Promise<string> {
    peer = new Peer();

    peer.on("connection", (conn) => {
        connections.push(conn);
        conn.on("close", () => {
            const index = connections.indexOf(conn);
            if (index !== -1) {
                connections.splice(index, 1);
            }
        });
    });

    return new Promise((resolve) => {
        peer!.once("open", () => {
            liveConnectionListeningStore.set(true);
            resolve(peer!.id);

            liveServerListeningAlert.set(true);
            setTimeout(() => {
                liveServerListeningAlert.set(false);
            }, 5000);
        });
    });
}
