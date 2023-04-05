export function isValidName(word: string){
    return /^\p{Lu}/u.test(word);
}