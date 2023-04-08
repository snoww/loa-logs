export function isValidName(word: string){
    return /^\p{Lu}/u.test(word);
}

export function removeUnknownHtmlTags(input: string){
    input = input.replace(/<\$TABLE_SKILLFEATURE[^>]*\/>/g, "??");
    input = input.replace(/<\$CALC[^>]*\/>/g, "??");
    return input;
}
  
  