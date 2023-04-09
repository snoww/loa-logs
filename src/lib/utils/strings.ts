import type { Entity } from "$lib/types";

export function isValidName(word: string){
    return /^\p{Lu}/u.test(word);
}

export function removeUnknownHtmlTags(input: string){
    input = input.replace(/<\$TABLE_SKILLFEATURE[^>]*\/>/g, "??");
    input = input.replace(/<\$CALC[^>]*\/>/g, "??");
    return input;
}
  
export function formatPlayerName(player: Entity,  hideNames = false): string {
    let playerName = player.name;
    // todo use settings
    if (!isValidName(playerName) || hideNames) {
        playerName = "";
        // playerName += " ("
        if (player.gearScore > 0) {
            playerName += player.gearScore + " ";
        }
        if (player.class) {
            playerName += player.class;
        }
        // playerName += ")";
    }
    if (player.isDead) {
        playerName = "💀 " + playerName;
    }

    return playerName;
}
