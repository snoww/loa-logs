export function getDateRange(pivot: Date, offsetWeeks = 0) {
    const date = new Date(pivot);
    date.setDate(date.getDate() + offsetWeeks * 7);

    const lastWednesday = new Date(date);
    const day = date.getDay();
    const diffToLastWed = (day + 4) % 7;
    lastWednesday.setDate(date.getDate() - diffToLastWed);
    const nextWednesday = new Date(lastWednesday);
    nextWednesday.setDate(lastWednesday.getDate() + 7);
    
    lastWednesday.setHours(10, 0, 0, 0);
    nextWednesday.setHours(10, 0, 0, 0);

    const format = (d: Date) => d.toLocaleDateString("en-GB").replace(/\//g, "-");
    
    return {
        dateFrom: lastWednesday.toISOString(),
        dateTo: nextWednesday.toISOString(),
        formatted: `${format(lastWednesday)} - ${format(nextWednesday)}`
    }
}

export function getBossImage(raidType: string) {
    switch(raidType) {
        case "echidnaG1":
            return "/images/raids/narkiel.jpg";
        case "echidnaG2":
            return "/images/raids/echidna.webp";
        case "behemothG1":
            return "/images/raids/behemoth.jpg";
        case "behemothG2":
            return "/images/raids/behemoth.jpg";
        case "act1G1":
            return "/images/raids/akkan.jpg";
        case "act1G2":
            return "/images/raids/aegir.webp";
        case "act2G1":
            return "/images/raids/narok.jpg";
        case "act2G2":
            return "/images/raids/act2-brel.webp";
        case "act3G1":
            return "/images/raids/thaemineinfernas.jpg";
        case "act3G2":
            return "/images/raids/naitreya.jpg";
        case "act3G3":
            return "/images/raids/mordum.png";
        case "drextalas":
            return "/images/raids/drextalas.webp";
        case "strikeG1":
            return "/images/raids/strike.webp";
        case "skolakia":
            return "/images/raids/skolakia.webp";
        case "argeos":
            return "/images/raids/argeos.webp";
    }
}