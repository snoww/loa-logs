export function stopPropagation(handler: (event: Event) => any) {
    return function (event: Event) {
        event.stopPropagation();
        handler(event);
    };
}

export function preventDefault(handler: (event: Event) => any) {
    return function (event: Event) {
        event.preventDefault();
        handler(event);
    };
}
